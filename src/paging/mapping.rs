use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::VirtAddr;


pub fn pt_map_page(
    mapper:&mut impl Mapper<Size4KiB>,
    frame_allocator:&mut impl FrameAllocator<Size4KiB>,
    page_range:PageRangeInclusive
)->Result<(),MapToError<Size4KiB>>{
    /*
    sets the entry on the page table
    for the specific pages included in the range
    !(inclusive)!
     */
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