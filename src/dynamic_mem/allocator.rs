use alloc::alloc::{GlobalAlloc,Layout};

#[global_allocator]
static ALLOCATOR:Allocator = Allocator;

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!()
    }
}
pub const HEAP_START:usize = 0x_aaaa_aaaa_0000;
pub const HEAP_SIZE: usize = 100 * 1024;