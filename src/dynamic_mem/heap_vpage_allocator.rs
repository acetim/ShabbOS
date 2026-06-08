use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::write;
use crate::dynamic_mem::allocator;
use crate::dynamic_mem::allocator::ALLOCATOR;

struct VpaNode {
    /*
    MUST BE SLAB ALLOCATOR SIZED!!!
     */
    start: usize,
    pages: usize,
    next: Option<*mut VpaNode>,
}
struct VirtualPageAllocator{
    next:Option<*mut VpaNode>
}
impl VirtualPageAllocator{
    pub const fn new()->Self{
        Self{next:None}
    }
    pub fn init(){

    }
    fn add_free_region(&mut self,start:usize,pages:usize) {
        let layout = Layout::new::<VpaNode>();
        unsafe{
            let vpa_node_ptr = ALLOCATOR.alloc(layout) as *mut VpaNode;
            write(vpa_node_ptr ,
                  VpaNode{
                        start,
                        pages,
                        next:self.next
                      }
            );
        }//assuming slab allocator is already ready



    }
}


