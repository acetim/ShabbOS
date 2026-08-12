use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::{MapToError, UnmapError};
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::VirtAddr;
use crate::paging::frame_allocator::FRAME_ALLOC;
use crate::paging::setup::KERNEL_PAGE_TABLE;


pub fn pt_unmap_page_range(start_page:usize, pages_to_free:usize) ->Result<(),UnmapError>{
    /*
    unmaps the entry on the page table
    for the specific pages included in the range
    !(inclusive)!
    &
    frees the frames from the FRAME ALLOCATOR BITMAP
     */
    let mapper = &mut *(KERNEL_PAGE_TABLE.wait().expect("kpt not initialized").lock());
    let frame_alloc = &mut *(FRAME_ALLOC.wait().expect("mapper is uninitialized").lock());
    //deallocate from bitmap
    for page in start_page..start_page+pages_to_free{
        let virt_addr = VirtAddr::new((page * 0x1000) as u64);
        let phys_frame =mapper
            .translate_page(Page::<Size4KiB>::containing_address(virt_addr))
            .expect("TRIED TO TRANSLATE UNMAPPED MEMORY");
        frame_alloc.free_frame(phys_frame);
    }
    
    let page_range= _get_page_range((start_page*0x1000) as u64, pages_to_free*0x1000);
    //unmap from kernel page table
    for page in page_range{
        mapper.unmap(page)?.1.flush();
    }    
    Ok(())
    
    
}

pub fn pt_map_page_range(start_page:u64, pages_to_allocate:usize) 
    ->Result<(),MapToError<Size4KiB>>{
    /*
    maps the entry on the page table
    for the specific pages included in the range
    !(inclusive)!
    &
    frees the frames from the FRAME ALLOCATOR BITMAP
     */
    let start_addr = start_page*0x1000;
    let size = pages_to_allocate *0x1000;
    let page_range = _get_page_range(start_addr, size);
    let frame_alloc = &mut *(FRAME_ALLOC.wait().expect("mapper is uninitialized").lock());
    let mapper = &mut *(KERNEL_PAGE_TABLE.wait().expect("kpt not initialized").lock());
    map_page_range_by_range(mapper, frame_alloc, page_range)?;
    Ok(())
}

fn map_page_range_by_range(
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

fn _get_page_range(addr:u64, size:usize) ->PageRangeInclusive{
    let heap_start = VirtAddr::new(addr);
    let heap_end =  VirtAddr::new((addr+size as u64)-1);
    let heap_start_page:Page<Size4KiB> = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);
    Page::range_inclusive(heap_start_page,heap_end_page)
}