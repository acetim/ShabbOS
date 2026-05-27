use uart_16550::{Config,Uart16550Tty,backend::PioBackend};
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref SERIAL1: Mutex<Uart16550Tty<PioBackend>> = Mutex::new (unsafe{
        Uart16550Tty::new_port(0x3f8, Config::default())
        .expect("uart init failed")
    });
}