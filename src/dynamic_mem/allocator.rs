
use alloc::alloc::{GlobalAlloc,Layout};
use core::mem;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;
use crate::{dbg, paging};
use crate::utils::locker::Locker;
use core::ptr::write;
use paging::frame_allocator::FRAME_ALLOC;
#[global_allocator]
static ALLOCATOR:Locker<KHeapAllocator> = Locker::new(KHeapAllocator::new());

pub enum KHeapSize{
    Size16bytes=16,
    Size32bytes=32,
    Size64bytes=64,
    Size128bytes=128,
    Size256bytes=256,
    Size512bytes=512,
    Size1024bytes=1024
}

struct KHeapSlot{
    next:Option<*mut KHeapSlot>,
}

#[derive(Debug)]
enum KHeapErr{
    InvalidSize
}
pub struct KHeapAllocator{
    cache:[Option<*mut KHeapSlot>;NUM_CACHES],
    expanded_pages_count:usize
}
pub const HEAP_START:usize = 0x_aaaa_aaaa_0000;
pub const NUM_CACHES:usize = 7;
pub const INIT_HEAP_SIZE: usize = NUM_CACHES * 0x1000;
pub const MIN_ALLOC_VAL:usize = 16;
//runs before KHeapAllocator::init()
pub fn kernel_heap_init(
    mapper:&mut impl Mapper<Size4KiB>,
    frame_allocator:&mut impl FrameAllocator<Size4KiB>
)->Result<(),MapToError<Size4KiB>>{
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end =  heap_start+ INIT_HEAP_SIZE -1u64;
        let heap_start_page:Page<Size4KiB> = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page,heap_end_page)
    };
    for page in page_range{
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::WRITABLE | PageTableFlags::PRESENT;
        unsafe{
            mapper.map_to(page,frame,flags,frame_allocator)?.flush();
        };
    }
    ALLOCATOR.lock().init();
    Ok(())

}



impl KHeapAllocator{
    pub const fn new()->KHeapAllocator{
        Self{
            cache:[None;NUM_CACHES],
            expanded_pages_count:0
        }
    }
    pub fn init(&mut self){
        for slab_idx in 0..NUM_CACHES{
            self.cache[slab_idx] = Self::split_page_to_slots(
                MIN_ALLOC_VAL<<slab_idx,
                HEAP_START+0x1000*slab_idx
            )
        }
    }

    //todo pentest ts shi vv
    fn split_page_to_slots(size:usize,addr:usize)->Option<*mut KHeapSlot>{
        /*
        takes a slot size and addr and converts
        the next 4kb to slab slots at the size of arg size
        returns a KHeapSlot head to the cache
         */
        dbg!("alignment of heap slot {}",mem::align_of::<KHeapSlot>());//todo check this
        dbg!("size of heap slot {}",mem::size_of::<KHeapSlot>());
        assert!(size>=mem::size_of::<KHeapSlot>());
        let head:*mut KHeapSlot = addr as *mut KHeapSlot;
        let num_slots = 0x1000/size;
        let mut temp = head;
        unsafe {
            for idx in 1..num_slots {
                write(temp, KHeapSlot { next: None });
                (*temp).next = Some((addr + size*idx) as *mut KHeapSlot);
                dbg!("created new slot at {}",addr + size*idx);
                temp = (*temp).next.expect("how did we get here?");
            }
            (*temp).next =None;
        }
        Some(head)
    }
    fn small_alloc(&mut self,layout: Layout)->*mut u8{
        //TODO verify min size!!!!!
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
                //todo expand and try again
            }

        }
        0 as *mut u8
    }
    fn expand_cache(&mut self,cache_idx:usize){
        let heap_end_addr=HEAP_START+INIT_HEAP_SIZE+(self.expanded_pages_count)*0x1000;
        let new_page:Page<Size4KiB> = Page::containing_address(VirtAddr::new(heap_end_addr as u64));
        //todo maybe handle this better vv
        let frame = FRAME_ALLOC.wait()
            .expect("error while trying to acquire frame allocator")
            .lock()
            .allocate_frame()
            .expect("physical frame allocation failed: no more memory available!!");
        
        let flags = PageTableFlags::WRITABLE | PageTableFlags::PRESENT;
        unsafe{
            (/*todo call active level 4 table and use as mapper*/).map_to(new_page,
                          frame,
                          flags,
                          FRAME_ALLOC.wait().expect("error while trying to acquire frame allocator").lock()
            )?.flush();
        };
    }
}






unsafe impl GlobalAlloc for Locker<KHeapAllocator>{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}
unsafe impl Send for KHeapAllocator {}