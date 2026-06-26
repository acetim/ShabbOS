
use alloc::alloc::{GlobalAlloc,Layout};
use core::mem;
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
use paging::mapping::pt_map_page;
use crate::dynamic_mem::heap_vpage_allocator::VirtualPageAllocator;
#[global_allocator]
pub static ALLOCATOR:Locker<KHeapAllocator> = Locker::new(KHeapAllocator::new());

struct KHeapSlot{
    //must be strictly under 16 bytes!
    next:Option<*mut KHeapSlot>,
}

pub struct KHeapAllocator{
    cache:[Option<*mut KHeapSlot>;NUM_CACHES],
    fallback_allocator: VirtualPageAllocator
}

pub const NUM_CACHES:usize = 7;
pub const MIN_ALLOC_VAL:usize = 16;

impl KHeapAllocator{
    pub const fn new()->KHeapAllocator{
        Self{
            cache:[None;NUM_CACHES],
            fallback_allocator:VirtualPageAllocator::new()
        }
    }
    //todo pentest ts shi vv
    fn split_page_to_slots(slot_size:usize, addr:usize) ->Option<*mut KHeapSlot>{
        /*
        takes a slot size and addr and converts
        the next 4kb to slab slots at the size of arg size
        returns a KHeapSlot head to the cache
         */
        dbg!("alignment of heap slot {}",mem::align_of::<KHeapSlot>());
        dbg!("size of heap slot {}",mem::size_of::<KHeapSlot>());
        assert!(slot_size >=mem::size_of::<KHeapSlot>());//todo remove this after testing
        let head:*mut KHeapSlot = addr as *mut KHeapSlot;
        let num_slots = 0x1000/ slot_size;
        let mut temp = head;
        unsafe {
            for idx in 1..num_slots {
                write(temp, KHeapSlot { next: None });
                (*temp).next = Some((addr + slot_size *idx) as *mut KHeapSlot);
                dbg!("created new slot at {}",addr + slot_size*idx);
                temp = (*temp).next.expect("how did we get here?");
            }
            (*temp).next =None;
        }
        Some(head)
    }

    fn cache_alloc(&mut self, layout: Layout) ->*mut u8{
        /*
        responsible to allocate small sized chunks
        (up to 1kb)
        returns a raw pointer to that chunk
        expands the cache when freelist is full
         */
        let size = layout.size();
        let cache_idx = (64-(size>>4).leading_zeros()) as usize;//strictly 64 bit !

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

    unsafe fn cache_free(&mut self, ptr: *mut u8, layout: Layout){
        let size = layout.size();
        let cache_idx = (64-(size>>4).leading_zeros()) as usize;
        unsafe{
            (*(ptr as *mut KHeapSlot)).next = self.cache[cache_idx]
        }
        self.cache[cache_idx]=Some(ptr as *mut KHeapSlot)
    }

    fn expand_cache(&mut self,cache_idx:usize)//todo physical cache page management
    -> Result<(),MapToError<Size4KiB>>{
        /*
        allocates a new 4kb phys frame for the kheap
        and breaks it down to caches
         */
        let new_page_addr =self.fallback_allocator.alloc_vpage(1) as usize;
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

    pub fn kalloc(&mut self,layout: Layout)->*mut u8{
        let alloc_size = layout.size();
        let max_slot_size= 16<<(NUM_CACHES-1);
        if (alloc_size<=max_slot_size){
            return self.cache_alloc(layout);
        }
        //fallback allocation
        let pages_to_allocate = alloc_size+(0xfff)/0x1000;
        let start_addr = self.fallback_allocator.alloc_vpage(pages_to_allocate) as u64;
        //map phys pages
        let page_range = {
            let heap_start = VirtAddr::new(start_addr);
            let heap_end =  VirtAddr::new((start_addr+alloc_size as u64)-1);
            let heap_start_page:Page<Size4KiB> = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page,heap_end_page)
        };
        let frame_alloc = &mut *(FRAME_ALLOC.wait().expect("mapper is uninitialized").lock());//todo create methods for ts
        let mapper = &mut *(KERNEL_PAGE_TABLE.wait().expect("kpt not initialized").lock());
        pt_map_page(mapper,frame_alloc,page_range).unwrap();
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
        //fallback allocator free
        let pages_to_free = (alloc_size+0xfff)/0x1000;
        self.fallback_allocator.free_vpage(ptr as usize,pages_to_free);
        //free phys frames
        let start_page = ptr as usize/0x1000;
        for page in start_page..start_page+pages_to_free{
            let virt_addr = VirtAddr::new((page * 0x1000) as u64);
            let phys_frame =KERNEL_PAGE_TABLE
                .wait()
                .unwrap()
                .lock()
                .translate_page(Page::<Size4KiB>::containing_address(virt_addr))
                .expect("TRIED TO FREE UNALLOCATED MEMORY");
            FRAME_ALLOC
                .wait()
                .unwrap()
                .lock()
                .free_frame(phys_frame);
        }
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