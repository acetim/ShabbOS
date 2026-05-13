use lazy_static::lazy_static;
use x86_64::registers::control::Cr2;
use crate::DEBUG;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::cpu_interrupts::{gdt, hardware};
use crate::dbg;
use crate::panic::hlt_loop;

lazy_static! {
    static ref IDT:InterruptDescriptorTable={
        let mut idt = InterruptDescriptorTable::new();

        unsafe{idt.double_fault.set_handler_fn(handler_double_fault).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)};
        idt.breakpoint.set_handler_fn(handler_breakpoint);
        idt.page_fault.set_handler_fn(handler_page_fault);

        idt[hardware::InterruptIndex::Timer.as_usize()]
            .set_handler_fn(hardware::handler_timer);
        idt[hardware::InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(hardware::handler_keyboard);

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
extern "x86-interrupt" fn handler_page_fault(interrupt_stack_frame: InterruptStackFrame,error_code:PageFaultErrorCode){
    use x86_64::registers::control::Cr2;
    dbg!("PAGE FAULT");
    dbg!("accessed addr: {:?}",Cr2::read());
    dbg!("Error Code: {:?}", error_code);
    dbg!("{:#?}", interrupt_stack_frame);
    hlt_loop();

}