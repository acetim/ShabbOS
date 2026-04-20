
#![no_std]
#![no_main]

use core::panic::PanicInfo;



static hello: &[u8] = b"hello world";

#[unsafe(no_mangle)]
pub extern "C" fn _start()->!{
    let vga_buffer = 0xb8000 as *mut u8;
    for(i,&byte) in hello.iter().enumerate(){
        unsafe {
            *vga_buffer.offset(i as isize *2) = byte;//char
            *vga_buffer.offset(i as isize *2+1) = 0xb;//color
        }
    }
    loop{}
}

#[panic_handler]
fn panic(_info:&PanicInfo)->!{
    loop{}
}

