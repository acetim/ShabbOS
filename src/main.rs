#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use bootloader::{entry_point, BootInfo};
use x86_64::structures::paging::Translate;
use x86_64::VirtAddr;
use crate::panic::hlt_loop;

mod macros;
mod serial;
mod tests;
mod cpu_interrupts;
mod vga;
mod testing;
mod panic;
mod paging;
mod dynamic_mem;

pub const DEBUG:bool = true;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo)->!{
    print_logo();
    init(boot_info);

    #[cfg(test)]
    test_main();

    let addresses=[0xb8000,0x201008,0x0100_0020_1a10,boot_info.physical_memory_offset];
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe{paging::setup::init(phys_mem_offset)};
    for address in &addresses{
        let virt = VirtAddr::new(*address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} - > {:?}",virt,phys)
    }
    println!("no crash!");
    hlt_loop()
}

fn init(boot_info: &'static BootInfo){
    paging::frame_allocator::init(&boot_info.memory_map,VirtAddr::new(boot_info.physical_memory_offset));
    cpu_interrupts::idt::idt_init();
    cpu_interrupts::gdt::gdt_init();
    unsafe{cpu_interrupts::hardware::PICS.lock().initialize();}
    x86_64::instructions::interrupts::enable();//sti

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

