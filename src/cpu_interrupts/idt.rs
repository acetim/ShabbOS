use lazy_static::lazy_static;
use crate::DEBUG;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::cpu_interrupts::gdt;
use crate::dbg;

lazy_static! {
    static ref IDT:InterruptDescriptorTable={
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(handler_breakpoint);
        unsafe{idt.double_fault.set_handler_fn(handler_double_fault).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)};
        idt
    };
}

pub fn idt_init(){
    IDT.load();
}

extern "x86-interrupt" fn handler_breakpoint( stack_frame:InterruptStackFrame){
    dbg!("EXCEPTION BREAKPOINT\n {:#?}",stack_frame);
    //todo handle
}
extern "x86-interrupt" fn handler_double_fault(stack_frame:InterruptStackFrame,error_code:u64)->!{
    panic!("double fault!\n {:#?}\n {}",stack_frame,error_code);
}