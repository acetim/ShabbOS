
use core::fmt;
use crate::serial::SERIAL1;
use x86_64::instructions::interrupts::without_interrupts;
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! dbg {
    () => (if(DEBUG){$crate::print!("\n")});
    ($($arg:tt)*) => (if(DEBUG){$crate::print!("{}\n", format_args!($($arg)*))});
}

#[macro_export]
macro_rules! print{
    ($($arg:tt)*)=> ($crate::macros::print(format_args!($($arg)*)))
}


#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::macros::serprint(format_args!($($arg)*));
    };
}
pub fn serprint(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(||{
        SERIAL1.lock().write_fmt(args).expect("printing to serial failed");
    });
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    without_interrupts(|| {
        crate::vga::vga_buffer::WRITER.lock().write_fmt(args).unwrap();
    });

}