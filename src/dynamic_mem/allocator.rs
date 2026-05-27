use alloc::alloc::{GlobalAlloc,Layout};
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;
use crate::utils::locker::Locker;
#[global_allocator]
static ALLOCATOR:Locker<KHeapAllocator> = Locker::new(KHeapAllocator::new());

pub struct KHeapAllocator{
    //todo
}

pub const HEAP_START:usize = 0x_aaaa_aaaa_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

//run before KHeapAllocator::init()
pub fn kernel_heap_init(
    mapper:&mut impl Mapper<Size4KiB>,
    frame_allocator:&mut impl FrameAllocator<Size4KiB>
)->Result<(),MapToError<Size4KiB>>{
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end =  heap_start+HEAP_SIZE-1u64;
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
    Ok(())

}
impl KHeapAllocator{
    pub const fn new()->Self{
        Self{}//todo
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