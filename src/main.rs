
#![no_std]
#![no_main]

mod vga_buffer;
mod macros;

use core::panic::PanicInfo;





#[unsafe(no_mangle)]
pub extern "C" fn _start()->!{
    println!("hello number {}",3);

    loop{}
}

#[panic_handler]
fn panic(_info:&PanicInfo)->!{
    loop{}
}

