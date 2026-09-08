#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use core::ops::DerefMut;
use bootloader::{entry_point, BootInfo};
use x86_64::structures::paging::{FrameAllocator, Translate};
use x86_64::VirtAddr;
use crate::dynamic_mem::allocator::ALLOCATOR;
use crate::panic::hlt_loop;
use crate::disk::pci::PciScanner;
mod macros;
mod tests;
mod cpu_interrupts;
mod vga;
mod testing;
mod panic;
mod paging;
mod dynamic_mem;
mod utils;
pub mod disk;

entry_point!(kernel_main);
//TODO attach a disk file to qemu, currently working on a null file (read/write will always retuen 0)
fn kernel_main(boot_info: &'static BootInfo)->!{
    print_logo();
    init(boot_info);
    #[cfg(test)]
    test_main();
    let mut scanner = PciScanner::new();
    scanner.check_all_busses_for_class((0x1,0x6));
    println!("no crash!");
    hlt_loop()
}

fn init(boot_info: &'static BootInfo){
    paging::frame_allocator::init(
        &boot_info.memory_map,
        VirtAddr::new(boot_info.physical_memory_offset)
    );
    cpu_interrupts::idt::idt_init();
    cpu_interrupts::gdt::gdt_init();
    unsafe{
        cpu_interrupts::hardware::PICS
            .lock()
            .initialize();
    }
    x86_64::instructions::interrupts::enable();//sti
    unsafe{
        paging::setup::init(
            VirtAddr::new(boot_info.physical_memory_offset)
        )
    };

    ALLOCATOR.lock().init();

}


fn print_logo(){
    println!(r#"
                            WELCOME         TO

                                                              9
                                 9          9 9             9 /
                                 |          \ | 9           |/ 9
         /@@@@@@   /@@@@@@     /\|-----.    /\|/   /\--.  /\|// /#\_  /\.
        /@@__  @@ /@ __@@@     |@@@@@@@@\   |@@@|  |@@@|  |@@@| |@@@| @@@\
       | @@  \ @@| @@  \__/     \@@@@@@@|    \@@/    @@/   \@@/ \@@/  \@@/
       | @@  | @@|  @@@@@@             ||      ||     @     /|  .''   //
       | @@  | @@ \____  @@            ||      ||    //     || //   ,//
       | @@  | @@ /@@  \ @@     _______||_     ||   /@      |@`/  ,/@/
       |  @@@@@@/|  @@@@@@/    /@@@@@@@@@@|    |@\@@@/      |@@@@@@@/
        \______/  \______/    |@@@@@@@@@@@  .oo@@@@@@       |@@@@@"
                               """"""""""   /@@@@@"'
                                            @@""'




                                            "#)
}

