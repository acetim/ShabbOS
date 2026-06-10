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
    freelist_head:Option<*mut VpaNode>
}
impl VirtualPageAllocator{
    pub const fn new()->Self{
        Self{ freelist_head:None}
    }
    pub fn init(){

    }
    fn add_free_region(&mut self,start_new:usize,pages_new:usize) {
        let layout = Layout::new::<VpaNode>();


        let mut current_node = self.freelist_head;
        let mut prev_node = None;
        while(current_node!=None){
            let start_current;
            unsafe{
                start_current=(*current_node
                    .expect("how did we get here"))
                    .start;
            }
            if(start_current<start_new){
                break;
            }
            prev_node = current_node;
            unsafe{current_node=(*current_node.unwrap()).next};
        }
        //atp current_node is before where start should be inserted

        let vpa_node_ptr;

        if(prev_node==None&&current_node==None){//no free nodes, set head as new node
            unsafe{
                vpa_node_ptr = ALLOCATOR.alloc(layout) as *mut VpaNode;
                write(vpa_node_ptr,VpaNode{start:start_new,pages:pages_new,next:None})
            }
            self.freelist_head = Some(vpa_node_ptr);
            return
        }
        if(prev_node==None){//add to start
            //todo add to start and expand
            return
        }
        if(current_node==None){
            //todo add to end and check merge
            return
        }
        //todo add to middle and check merge



    }
}


