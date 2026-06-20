use alloc::alloc::dealloc;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::write;
use crate::dynamic_mem::allocator::ALLOCATOR;
use crate::dynamic_mem::heap_errors::HeapErr;
use crate::dynamic_mem::heap_errors::HeapErr::OutOfSpace;

struct VpaNode {
    /*
    MUST BE SLAB ALLOCATOR SIZED!!!
     */
    start: usize,
    pages: usize,
    next: Option<*mut VpaNode>,
}
impl VpaNode{
    pub fn end(&self)->usize{
        self.start+self.pages*0x1000
    }
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
    fn alloc_vpage(&mut self,pages_to_allocate:usize)->Result<*mut u8,HeapErr>{//todo check this
        /*
        takes an amount of pages to allocate
        returns a pointer to an area of that size
        MUST NOT USE alloc() inside this function and its callees
        or infinite recursion will be possible
         */
        let layout = Layout::new::<VpaNode>();
        let mut cur_node = self.freelist_head;
        let mut prev_node = None;
        let mut allocated_addr:*mut u8;
        while let Some(cur_ptr)= cur_node {
            unsafe {
                if (*cur_ptr).pages >= pages_to_allocate {

                    (*cur_ptr).pages -= pages_to_allocate;
                    allocated_addr =(*cur_ptr).start as *mut u8;
                    (*cur_ptr).start +=pages_to_allocate*0x1000;

                    if((*cur_ptr).pages==0){
                       if let Some(prev_node)= cur_node {
                           (*prev_node).next=(*cur_ptr).next;
                           ALLOCATOR.dealloc(cur_ptr as *mut u8, layout);

                       }
                       else{
                           self.freelist_head = (*cur_ptr).next;
                           ALLOCATOR.dealloc(cur_ptr as *mut u8, layout);
                       }

                    }
                    return Ok(allocated_addr);
                }
                prev_node = cur_node;
                cur_node =(*cur_ptr).next;
            }
        }
        Err(OutOfSpace)
    }
    fn add_free_region(&mut self,start_new:usize,pages_new:usize) {
        /*
        this function takes a new addr and a number of pages
        it expands/adds nodes to the freelist
        ...<-current<-new<-prev<-...
         */
        let mut current_node = self.freelist_head;
        let mut prev_node = None;
        let new_node_ref;
        unsafe{
            new_node_ref = &mut *Self::create_vpa_node(start_new, pages_new, None).expect("how did we get here")
        }
        //go to new node position
        while let Some(current_ptr)=current_node {
            unsafe{
                if (*current_ptr).start > start_new {
                    break;
                }
                prev_node = current_node;
                current_node = (*current_ptr).next;
            }
        }
        //insert
        if let Some(prev_ptr) = prev_node {
            unsafe {
                (*prev_ptr).next = Some(new_node_ref as *mut VpaNode);
            }
        }
        else{//insert as head
            self.freelist_head=Some(new_node_ref as *mut VpaNode);
        }
        new_node_ref.next = current_node;
        //merge
        Self::merge_three(prev_node,new_node_ref,current_node);
    }
    fn merge_three(prev:Option<*mut VpaNode>,mut mid:&mut VpaNode,next:Option<*mut VpaNode>){
        /*
        merges the nodes if next to each other
        always merges nodes to the right
         */
        //todo vr ts
        let layout = Layout::new::<VpaNode>();
        if let Some(prev_node)=prev{
            unsafe{
                if(*prev_node).end()==mid.start{
                    //merge mid to prev and set prev as new mid
                    (*prev_node).pages+=mid.pages;
                    (*prev_node).next = next;
                    ALLOCATOR.dealloc(mid as *mut VpaNode as *mut u8,layout);
                    mid = &mut *prev_node;//not uaf lol
                }
            }
        }
        if let Some(next_node)=next{
            unsafe{
                if(*next_node).start==mid.end(){
                    mid.pages+=(*next_node).pages;
                    mid.next=(*next_node).next;
                    ALLOCATOR.dealloc(next_node as *mut u8,layout);
                }
            }
        }


    }
    fn create_vpa_node(start:usize,pages:usize,next:Option<*mut VpaNode>)
        ->Option<*mut VpaNode>
    {
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


