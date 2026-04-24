#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
mod macros;
mod serial;
mod tests;
mod cpu_interrupts;
mod vga;
mod testing;
pub const DEBUG:bool = true;

#[unsafe(no_mangle)]
pub extern "C" fn _start()->!{
    print_logo();
    init();

    #[cfg(test)]
    test_main();

    println!("succsessful!");
    loop{}
}

fn init(){
    cpu_interrupts::idt::idt_init();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info:&PanicInfo)->!{
    println!("{}",info);
    loop{}
}
fn print_logo(){
    println!(r#"
                            WELCOME         TO

                                                              9
                                 9          9 9             9 /
                                 |          \ | 9           |/ 9
         /@@@@@@   /@@@@@@     /\|-----.    /\|/   /\--.  /\|// /#\_  /\.
        /@@__  @@ /@ __@@@\    |@@@@@@@@\   |@@@|  |@@@|  |@@@| |@@@| @@@\
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

