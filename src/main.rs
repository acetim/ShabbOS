#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
mod vga_buffer;
mod macros;
mod serial;
mod tests;
use core::panic::PanicInfo;
use x86_64::instructions::port::Port;

const DEBUG:bool = true;
#[unsafe(no_mangle)]
pub extern "C" fn _start()->!{
    print_logo();
    #[cfg(test)]
    test_main();
    loop{}
}
#[cfg(not(test))]
#[panic_handler]
fn panic(info:&PanicInfo)->!{
    println!("{}",info);
    loop{}
}
fn print_logo(){
    println!(r" __      __  ____    __       ____     _____            ____
/\ \  __/\ \/\  _`\ /\ \     /\  _`\  /\  __`\  /'\_/`\/\  _`\
\ \ \/\ \ \ \ \ \L\_\ \ \    \ \ \/\_\\ \ \/\ \/\      \ \ \L\_\
 \ \ \ \ \ \ \ \  _\L\ \ \  __\ \ \/_/_\ \ \ \ \ \ \__\ \ \  _\L
  \ \ \_/ \_\ \ \ \L\ \ \ \L\ \\ \ \L\ \\ \ \_\ \ \ \_/\ \ \ \L\ \
   \ `\___x___/\ \____/\ \____/ \ \____/ \ \_____\ \_\\ \_\ \____/
 _____________  \/___/  \/___/   \/___/   \/_____/\/_/ \/_/\/___/
/\__  _\/\  __`\
\/_/\ \/\ \ \/\ \
   \ \ \ \ \ \ \ \
    \ \ \ \ \ \_\ \
     \ \_\ \ \_____\
 ____ \/___ \/_____/       __       __       _____   ____         __
/\  _`\ /\ \              /\ \     /\ \     /\  __`\/\  _`\      /\ \
\ \,\L\_\ \ \___      __  \ \ \____\ \ \____\ \ \/\ \ \,\L\_\    \ \ \
 \/_\__ \\ \  _ `\  /'__`\ \ \ '__`\\ \ '__`\\ \ \ \ \/_\__ \     \ \ \
   /\ \L\ \ \ \ \ \/\ \L\.\_\ \ \L\ \\ \ \L\ \\ \ \_\ \/\ \L\ \    \ \_\
   \ `\____\ \_\ \_\ \__/.\_\\ \_,__/ \ \_,__/ \ \_____\ `\____\    \/\_\
    \/_____/\/_/\/_/\/__/\/_/ \/___/   \/___/   \/_____/\/_____/     \/_/      ")
}

/////////////////////////////////////////////////////////////////////////TESTING!!!!
#[cfg(test)]
#[panic_handler]
fn panic(info:&PanicInfo)->!{
    serial_println!("\n[failed]");
    serial_println!("{}",info);
    exit_qemu(QemuExitCodes::Falied);
    loop {}
}
#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]){
    serial_println!("running {} tests",tests.len());
    for test in tests {
        test.run();
    }
    serial_println!("all tests passed");
    exit_qemu(QemuExitCodes::Success);


}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[repr(u32)]
pub enum QemuExitCodes{
    Success = 0x10,
    Falied = 0x11,
}
pub fn exit_qemu(exit_code:QemuExitCodes){
    use x86_64::instructions::port::Port;
    unsafe{
        let mut port= Port::new(0xf4);//qemu iobase port
        port.write(exit_code as u32)
    }
}
pub trait Testable {
    fn run(&self) -> ();
}
impl <T> Testable for T
where
    T:Fn()
{
    fn run(&self){
        serial_print!("{}...\t",core::any::type_name::<T>());
        self();
        serial_println!("[OK]");
    }
}