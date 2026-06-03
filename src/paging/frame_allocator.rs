use x86_64::structures::paging::{FrameAllocator, PhysFrame};

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use x86_64::{PhysAddr, VirtAddr};
use crate::{dbg, println};
use spin::{Mutex, Once};
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::structures::paging::Size4KiB;
//TODO add a separate optimized func for multiple frame requests at a time
//TODO REVIEW INIT FOR BUGS
//TODO REMOVE DBG!
pub static FRAME_ALLOC: Once<Mutex<BitmapFrameAllocator>> = Once::new();

pub fn get_frame_allocator()->&'static Mutex<BitmapFrameAllocator>{
    FRAME_ALLOC.wait().expect("failed while trying to get frame allocator")
}
pub fn init(memory_map:&'static MemoryMap,phys_mem_offset:VirtAddr){
    FRAME_ALLOC.call_once(||{
        Mutex::new(BitmapFrameAllocator::new(memory_map,phys_mem_offset))
    });
}
pub struct BitmapFrameAllocator{
    bitmap:&'static mut [u64]
}
impl BitmapFrameAllocator{

    pub fn new(memory_map:&'static MemoryMap,phys_mem_offset:VirtAddr)->Self{
        /*
        initializes the bitmap, sets each bit to 1 of page is used and 0 if free
         */
        let max_phys_addr = memory_map.iter().map(|r| r.range.end_addr()).max().unwrap();
        let total_frames = (max_phys_addr+4095)/4096;
        let bitmap_size_qwords = ((total_frames+63)/64) as usize;
        dbg!("need to allocate {} qwords for bitmap",bitmap_size_qwords);

        let mut bitmap_phys_addr: u64 =0;
        for region in memory_map.iter().filter(|r| r.region_type == MemoryRegionType::Usable){
            let start = region.range.start_addr();
            let end = region.range.end_addr();
            if((end-start)>= (bitmap_size_qwords*8) as u64 ){
                bitmap_phys_addr= start;
                break;
            }
        }
        if bitmap_phys_addr == 0 {
            panic!("could not find a free memory region to hold the frame bitmap!");
        }
        dbg!("found clear memory at {:x} ",bitmap_phys_addr);

        let bitmap_virt_addr = phys_mem_offset + bitmap_phys_addr;
        dbg!("translated {:x} to virtual memory at{:?} ",bitmap_phys_addr,bitmap_virt_addr);

        let bitmap: &mut [u64] = unsafe {
            core::slice::from_raw_parts_mut(bitmap_virt_addr.as_mut_ptr(), bitmap_size_qwords)
        };
        bitmap.fill(0xff);
        for region in memory_map.iter().filter(|r| r.region_type == MemoryRegionType::Usable){
            let start_frame = region.range.start_frame_number;
            let end_frame = region.range.end_frame_number;//ths frame no longer belongs to the region
            dbg!("clearing frames {} -> {}",start_frame,end_frame);
            for frame_index in start_frame..end_frame{
                let index = frame_index as usize ;
                bitmap[index/64]&=!(1u64<<index%64);
            }
        }

        let bitmap_start_frame = bitmap_phys_addr / 4096;
        let bitmap_end_frame = (bitmap_phys_addr + bitmap_size_qwords as u64 + 4095) / 4096;
        dbg!("clearing frames reserved for bitmap {} -> {}",bitmap_start_frame ,bitmap_end_frame);
        for frame_index in bitmap_start_frame..bitmap_end_frame{
            let index = frame_index as usize ;
            bitmap[index/64]|=1u64<<index%64;
        }
        Self{bitmap}
    }
}
unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) ->Option<PhysFrame>{
        /*
        "hardware accelerated" frame allocator with TZCNT!!!
        uses the bitmap to determine the first free page
         */
        without_interrupts(||{
            for (idx,qword_pages) in self.bitmap.iter_mut().enumerate(){
                if(*qword_pages!=u64::MAX){

                    let page_inner=(*qword_pages).trailing_ones();
                    *qword_pages|=(1u64<<page_inner);

                    let page_index = ((idx*64)+(page_inner as usize)) as u64;
                    return Some(PhysFrame::containing_address(PhysAddr::new(page_index*4096)))
                }
            }
            None
        })
    }
}
unsafe impl Send for BitmapFrameAllocator {}
unsafe impl Sync for BitmapFrameAllocator {}