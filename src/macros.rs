
use core::fmt;


#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
#[macro_export]
macro_rules! print{
    ($($arg:tt)*)=> ($crate::macros::print(format_args!($($arg)*)))
}

pub fn print(args: fmt::Arguments) {
    use core::fmt::Write;
    crate::vga_buffer::WRITER.lock().write_fmt(args).unwrap();
}