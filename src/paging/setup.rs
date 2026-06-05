use bootloader::bootinfo::MemoryMap;
use spin::{Mutex, Once};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::page_table::FrameError;
use crate::paging::frame_allocator::{BitmapFrameAllocator, FRAME_ALLOC};

pub static KERNEL_PAGE_TABLE: Once<Mutex<OffsetPageTable<'static>>> = Once::new();

pub unsafe fn init(phys_mem_offset:VirtAddr){
    /*
    caller must ensure valid phys mem offset
     */
    let level_4_table = active_level_4_table(phys_mem_offset);
    KERNEL_PAGE_TABLE.call_once(||{
        Mutex::new(OffsetPageTable::new(level_4_table,phys_mem_offset))
    });
}






unsafe fn active_level_4_table(phys_mem_offset:VirtAddr) ->&'static mut PageTable{
    /*
    returns the active page table stored in cr3
     */
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame,_)=Cr3::read();

    let phys =level_4_table_frame.start_address();
    let virt = phys_mem_offset+phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe {&mut *page_table_ptr}
}


