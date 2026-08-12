
use alloc::alloc::{GlobalAlloc,Layout};
use core::ops::DerefMut;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags,Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;
use crate::{dbg, paging};
use crate::utils::locker::Locker;
use core::ptr::write;
use paging::frame_allocator::FRAME_ALLOC;
use paging::setup::KERNEL_PAGE_TABLE;
use paging::frame_allocator::get_frame_allocator;
use crate::paging::mapping::{pt_unmap_page_range, pt_map_page_range};

pub const HEAP_SIZE: usize = 100 * 1024;
pub const HEAP_START:usize = 0x_4444_4444_0000;
#[global_allocator]
pub static ALLOCATOR:Locker<KHeapAllocator> = Locker::new(KHeapAllocator::new());



// Ensure this trait is imported!
struct VpaNode {
    /*
    MUST BE SLAB ALLOCATOR SIZED!!!
     */
    start: usize,
    pages: usize,
    next: Option<*mut VpaNode>,
}
impl VpaNode {
    pub fn end(&self)->usize{
        self.start+self.pages*0x1000
    }
}

struct KHeapSlot{
    //must be strictly under 16 bytes!
    next:Option<*mut KHeapSlot>,
}

pub struct KHeapAllocator{
    cache:[Option<*mut KHeapSlot>;NUM_CACHES],
    freelist_head:Option<*mut VpaNode>,
    bootstrap_vpa_node:VpaNode
}

pub const NUM_CACHES:usize = 7;
pub const MIN_ALLOC_VAL:usize = 16;

impl KHeapAllocator{
    pub const fn new()->KHeapAllocator{
        Self{
            cache:[None;NUM_CACHES],
            freelist_head:None,
            bootstrap_vpa_node:VpaNode{
                start:HEAP_START,
                pages:HEAP_SIZE/0x1000,
                next:None
            }
        }
    }

    pub fn init(&mut self){//BOOTSTRAPPPPPPPP
        /*
        initializes the heap and sets the first vpa node in the freelist
         */
        dbg!("stating kernel heap init...");
        dbg!("assigning bootstrap node as freelist head");
        self.freelist_head = Some(&mut self.bootstrap_vpa_node as *mut VpaNode);
        dbg!("allocating space for new initial vpa node");
        let heap_vpa=self.cache_alloc(Layout::new::<VpaNode>()) as *mut VpaNode;
        dbg!("copying contents of old vpa node");
        unsafe{
            (*heap_vpa).start=(*self.freelist_head.unwrap()).start;
            (*heap_vpa).pages=(*self.freelist_head.unwrap()).pages;
            (*heap_vpa).next=None;
        }
        dbg!("setting new head as new node");
        self.freelist_head=Some(heap_vpa);
        dbg!("heap initialization done!");

    }
    //todo tst
    fn split_page_to_slots(slot_size:usize, addr:usize) ->Option<*mut KHeapSlot>{
        /*
        takes a slot size and addr and converts
        the next 4kb to slab slots at the size of arg size
        returns a KHeapSlot head to the cache
         */
        assert!(slot_size >=size_of::<KHeapSlot>());//todo remove this after testing
        let head:*mut KHeapSlot = addr as *mut KHeapSlot;
        let num_slots = 0x1000/ slot_size;
        let mut temp = head;
        unsafe {
            for idx in 1..num_slots {//todo check +1 bugs
                let new_node = KHeapSlot{next:None};
                *temp = new_node;
                (*temp).next = Some((addr + slot_size *idx) as *mut KHeapSlot);
                temp = (*temp).next.expect("how did we get here?");
            }
            (*temp).next =None;
        }
        Some(head)
    }

    pub fn cache_alloc(&mut self, layout: Layout) ->*mut u8{
        /*
        responsible to allocate small sized chunks
        (up to 1kb)
        returns a raw pointer to that chunk
        expands the cache when freelist is full
         */
        let size = layout.size();
        let cache_idx = Self::_get_cache_idx(size);//strictly 64 bit !

        match self.cache[cache_idx]{
            Some(chunk) => {
                unsafe {
                    self.cache[cache_idx] = (*chunk).next;
                }
                return chunk as *mut u8;
            },
            None=>{
                match self.expand_cache(cache_idx) {
                    Err(e)=>{panic!("{:?}",e)} //page table error
                    _=>{}
                }
                let new_chunk = self.cache[cache_idx].expect("how DID we get here???");
                unsafe {
                    self.cache[cache_idx] = (*new_chunk).next;
                }
                return new_chunk as *mut u8;
            }

        }
    }

    pub unsafe fn cache_free(&mut self, ptr: *mut u8, layout: Layout){
        let size = layout.size();
        let cache_idx = Self::_get_cache_idx(size);
        unsafe{
            (*(ptr as *mut KHeapSlot)).next = self.cache[cache_idx]
        }
        self.cache[cache_idx]=Some(ptr as *mut KHeapSlot)
    }

    fn expand_cache(&mut self,cache_idx:usize)//todo better physical cache page management
    -> Result<(),MapToError<Size4KiB>>{
        /*
        allocates a new 4kb phys frame for the kheap
        and breaks it down to caches
         */
        let new_page_addr =self._fallback_alloc_vpage(1) as usize;
        let new_page:Page<Size4KiB> = Page::containing_address(VirtAddr::new(new_page_addr as u64));
        //todo maybe handle this better vv
        let new_frame = FRAME_ALLOC.wait()
            .expect("error while trying to acquire frame allocator")
            .lock()
            .allocate_frame()
            .expect("physical frame allocation failed: no more memory available!!");

        let flags = PageTableFlags::WRITABLE | PageTableFlags::PRESENT;
        let mut mapper =KERNEL_PAGE_TABLE
            .wait()
            .expect("kernel page table has not been initialized")
            .lock();
        unsafe{
            (*mapper).map_to(
                new_page,
                new_frame,
                flags,
                get_frame_allocator().lock().deref_mut()
            )?.flush();
        };
        let slot_size = 16<<cache_idx;

        self.cache[cache_idx]=Self::split_page_to_slots(slot_size,new_page_addr);
        Ok(())
    }
    #[inline]
    fn _get_cache_idx(size: usize) -> usize {
        let size = size.max(MIN_ALLOC_VAL);
        ((64-(size-1).leading_zeros()) as usize-4).min(NUM_CACHES-1)
    }

    pub fn _fallback_alloc_vpage(&mut self, pages_to_allocate:usize) ->*mut u8{//todo check this
        /*
        takes an amount of virtual pages to allocate
        returns a pointer to an area of that size
        MUST NOT USE alloc() inside this function and its callees
        or infinite recursion will be possible
         */
        let layout = Layout::new::<VpaNode>();
        let mut cur_node = self.freelist_head;
        let mut prev_node = None;
        let mut allocated_addr:*mut u8;
        while let Some(cur_ptr)= cur_node {
            unsafe {
                if (*cur_ptr).pages >= pages_to_allocate {

                    (*cur_ptr).pages -= pages_to_allocate;

                    allocated_addr =(*cur_ptr).start as *mut u8;
                    (*cur_ptr).start +=pages_to_allocate*0x1000;

                    if((*cur_ptr).pages==0){
                        if let Some(prev_node)= cur_node {
                            (*prev_node).next=(*cur_ptr).next;
                        }
                        else{
                            self.freelist_head = (*cur_ptr).next;
                        }
                        self.cache_free(cur_ptr as *mut u8, layout);
                    }
                    return allocated_addr;
                }
                prev_node = cur_node;
                cur_node =(*cur_ptr).next;
            }
        }

        panic!("damn now i need to implement heap expansion");

    }

    pub fn _fallback_free_vpage(&mut self, start:usize, pages:usize){
        self._fallback_add_free_vpage_region(start, pages);
    }

    fn _fallback_add_free_vpage_region(&mut self, start_new:usize, pages_new:usize) {
        /*
        this function takes a new addr and a number of pages
        it expands/adds nodes to the freelist
        ...<-current<-new<-prev<-...
         */
        let mut current_node = self.freelist_head;
        let mut prev_node = None;
        let new_node_ref;
        unsafe{
            new_node_ref = &mut *self._fallback_create_vpa_node(start_new,pages_new, None).expect("how did we get here")
        }
        //go to new node position
        while let Some(current_ptr)=current_node {
            unsafe{
                if (*current_ptr).start > start_new {
                    break;
                }
                prev_node = current_node;
                current_node = (*current_ptr).next;
            }
        }
        //insert
        if let Some(prev_ptr) = prev_node {
            unsafe {
                (*prev_ptr).next = Some(new_node_ref as *mut VpaNode);
            }
        }
        else{//insert as head
            self.freelist_head=Some(new_node_ref as *mut VpaNode);
        }
        new_node_ref.next = current_node;
        //merge
        self._fallback_merge_three(prev_node, new_node_ref, current_node);
    }

    fn _fallback_merge_three(&mut self, prev:Option<*mut VpaNode>, mut mid:&mut VpaNode, next:Option<*mut VpaNode>){
        /*
        merges the nodes if next to each other
        always merges nodes to the right
         */
        //todo vr ts
        let layout = Layout::new::<VpaNode>();
        if let Some(prev_node)=prev{
            unsafe{
                if(*prev_node).end()==mid.start{
                    //merge mid to prev and set prev as new mid
                    (*prev_node).pages+=mid.pages;
                    (*prev_node).next = next;
                    self.cache_free(mid as *mut VpaNode as *mut u8,layout);
                    mid = &mut *prev_node;//not uaf lol
                }
            }
        }
        if let Some(next_node)=next{
            unsafe{
                if(*next_node).start==mid.end(){
                    mid.pages+=(*next_node).pages;
                    mid.next=(*next_node).next;
                    self.cache_free(next_node as *mut u8,layout);
                }
            }
        }


    }

    fn _fallback_create_vpa_node(&mut self, start:usize, pages:usize, next:Option<*mut VpaNode>)
        ->Option<*mut VpaNode>
    {
        let layout = Layout::new::<VpaNode>();
        let vpa_node_ptr;
        unsafe{
            vpa_node_ptr = self.cache_alloc(layout) as *mut VpaNode;
            write(vpa_node_ptr,
                  VpaNode {
                      start,
                      pages,
                      next
                  }
            )
        }
        Some(vpa_node_ptr)
    }
    #[inline]
    fn _fallback_bytes_to_pages(alloc_size:usize)->usize{
        (alloc_size+(0xfff))/0x1000
    }
    pub fn kalloc(&mut self,layout: Layout)->*mut u8{

        let alloc_size = layout.size();
        let max_slot_size= 16<<(NUM_CACHES-1);
        if (alloc_size<=max_slot_size){
            return self.cache_alloc(layout);
        }
        //fallback allocation
        let pages_to_allocate = Self::_fallback_bytes_to_pages(alloc_size);
        //allocate
        let start_addr = self._fallback_alloc_vpage(pages_to_allocate) as u64;

        //map
        pt_map_page_range(
            start_addr/0x1000,
            Self::_fallback_bytes_to_pages(alloc_size)
        ).unwrap();

        start_addr as *mut u8
    }

    pub unsafe fn kfree(&mut self,ptr: *mut u8,layout: Layout){//todo check off by 1 errs
        //cache free
        let alloc_size = layout.size();
        let max_slot_size= 16<<(NUM_CACHES-1);
        if (alloc_size<=max_slot_size){
            self.cache_free(ptr,layout);
            return
        }
        let pages_to_free = Self::_fallback_bytes_to_pages(alloc_size);
        //deallocate
        self._fallback_free_vpage(ptr as usize, pages_to_free);
        //unmap
        let start_page = ptr as usize/0x1000;
        pt_unmap_page_range(start_page, pages_to_free).unwrap();
    }

}

unsafe impl GlobalAlloc for Locker<KHeapAllocator>{

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().kalloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().kfree(ptr,layout)
    }
}
unsafe impl Send for KHeapAllocator {}