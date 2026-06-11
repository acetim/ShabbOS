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
        /*
        this function takes a new addr and a number of pages
        it expands/adds nodes to the freelist
         */
        let mut current_node = self.freelist_head;
        let mut prev_node = None;
        //go to new node position
        while(current_node!=None){
            let current_ref;
            unsafe{
                current_ref=&*current_node
                    .expect("how did we get here");
            }
            if(current_ref.start<start_new){
                break;
            }
            prev_node = current_node;
            current_node=current_ref.next;
        }
        //no free nodes, set head as new node
        if(prev_node==None&&current_node==None){
            self.freelist_head = Self::create_vpa_node(
                start_new,
                pages_new,
                None
            );
            return
        }
        //add/merge node to start
        if(prev_node==None){
            let current_ref = unsafe{
                &mut (*current_node.unwrap())
            };
            if(start_new+(pages_new*0x1000)==current_ref.start){//merge
                current_ref.start=current_ref.start-(pages_new*0x1000);
                return;
            }
            self.freelist_head = Self::create_vpa_node(
                start_new,
                pages_new,
                self.freelist_head
            );
            return
        }
        //add/merge node to end
        if(current_node==None){
            let prev = unsafe{
                &mut (*prev_node.unwrap())
            };
            if(prev.start+(prev.pages*0x1000)==start_new){//merge
                prev.pages+=pages_new;
                return
            }
            prev.next = Self::create_vpa_node(
                start_new,
                pages_new,
                None
            );
            return
        }
        //todo add to middle and check merge
    }
    fn create_vpa_node(start:usize,pages:usize,next:Option<*mut VpaNode>)->Option<*mut VpaNode>{
        let layout = Layout::new::<VpaNode>();
        let vpa_node_ptr;
        unsafe{
            vpa_node_ptr = ALLOCATOR.alloc(layout) as *mut VpaNode;
            write(vpa_node_ptr,
                  VpaNode {
                      start,
                      pages,
                      next
                  }
            )
        }
        Some(vpa_node_ptr)
    }
}


