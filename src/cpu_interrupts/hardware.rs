use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;
use crate::print;
use pc_keyboard::{layouts,DecodedKey,HandleControl,Keyboard,ScancodeSet1};
use x86_64::instructions::port::Port;

pub const PIC_START:u8 = 32;
pub const PIC_END:u8 = PIC_START+8;


pub static PICS:spin::Mutex<ChainedPics>=spin::Mutex::new(
    unsafe {ChainedPics::new(PIC_START,PIC_END)}
);
#[derive(Debug,Clone,Copy)]
#[repr(u8)]
pub enum InterruptIndex{
    Timer = PIC_START,
    Keyboard = PIC_START+1
}
impl InterruptIndex{
    pub fn as_u8(self) -> u8{
        self as u8
    }
    pub fn as_usize(self)->usize{
        usize::from(self.as_u8())
    }
}

///////////////////////////////handlers
pub extern "x86-interrupt" fn handler_timer(interrupt_stack_frame: InterruptStackFrame){
    //todo handle
    unsafe{PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());}
}
pub extern "x86-interrupt" fn handler_keyboard(interrupt_stack_frame: InterruptStackFrame){

    let mut port = Port::new(0x60);
    let scancode:u8 = unsafe{port.read()};
    print_scancode(scancode);

    unsafe {PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());}
}
lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

pub fn print_scancode(scancode:u8){

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode){
        if let Some(key) = keyboard.process_keyevent(key_event){
            match key {
                DecodedKey::Unicode(character) =>print!("{}",character),
                DecodedKey::RawKey(key)=>print!("{:?}",key)
            }
        }
    }

}