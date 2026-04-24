use core::panic::PanicInfo;
use crate::{serial_print, serial_println};

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