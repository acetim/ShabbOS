use lazy_static::lazy_static;
use crate::DEBUG;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::dbg;

lazy_static! {
    static ref IDT:InterruptDescriptorTable={
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(handler_breakpoint);
        idt
    };
}

pub fn idt_init(){
    IDT.load();
}

extern "x86-interrupt" fn handler_breakpoint( stack_frame:InterruptStackFrame){
    dbg!("EXCEPTION BREAKPOINT\n {:#?}",stack_frame);
    
}