//static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

// the x86_64 crate has built-in abstractions that allow us to create IDT that will map
// CPU exceptions to interrupt handlers
use crate::gdt;
use crate::println;
use crate::print;
use x86_64::structures::idt::{ InterruptDescriptorTable, InterruptStackFrame };
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}



pub static PICS:  Mutex<ChainedPics> = Mutex::new(
    unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    }
);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        // creating an IDT that we can add interrupt handlers to
        let mut idt = InterruptDescriptorTable::new();
        // setting the breakpoint handler
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        // setting a double fault handler
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        //idt[32].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

// x86_64 arch does not allow returning from a double fault so
// the exception handler should diverge ( -> !)
// the _error_code is always 0
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT \n {:#?}", stack_frame);
}



// a breakpoint interrupt handler that use the x86-interrupt calling convention
// that simply prints text and the stack frame
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    // invoking a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    print!(".");

    unsafe {
        PICS.lock()
        .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
