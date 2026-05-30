
use alloc::alloc::{GlobalAlloc,Layout};
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;
use crate::dbg;
use crate::utils::locker::Locker;
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
    next:Option<*mut KHeapSlot>, //8 bytes
}


enum KHeapErr{
    InvalidSize
}
pub struct KHeapAllocator{
    cache:[Option<*mut KHeapSlot>;7]
}
pub const HEAP_START:usize = 0x_aaaa_aaaa_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

//runs before KHeapAllocator::init()
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
    //todo pentest ts shi
    pub fn split_page_to_slots(size:usize,addr:usize)->Result<Option<*mut KHeapSlot>,KHeapErr>{
        /*
        takes a slot size and addr and converts
        the next 4kb to slab slots at the size of arg size
        returns a KHeapSlot head to the cache
         */
        if size < 16 {return Err(KHeapErr::InvalidSize);}
        let head:*mut KHeapSlot = addr as *mut KHeapSlot;
        let num_slots = 0x1000/size;
        unsafe {
            let mut temp = head;
            for idx in 1..num_slots {
                (*temp).next = Some((addr + size*idx) as *mut KHeapSlot);
                dbg!("created new slot at {}",addr + size*idx);
                temp = (*temp).next.expect("how did we get here?");//using unwrap because im sure theres an addr there
            }
            (*temp).next =None;
        }
        Ok(Some(head))
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