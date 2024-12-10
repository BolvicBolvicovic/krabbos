use crate::{println, tables::InterruptStackFrame};

pub extern "x86-interrupt" fn divide_error(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn debug(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn non_maskable_interrupt(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn bound_range_exceeded(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_opcode(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OP CODE\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn coprocessor_not_available(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: COPROCESSOR NOT AVAILABLE\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_tss(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn segment_not_present(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn stack_segment_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn general_protection_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: GPF\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
}
pub extern "x86-interrupt" fn x87_floating_point(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: x87_floating_point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_check(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: alignment_check\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn machine_check(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: machine_check\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_floating_point(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: simd_floating_point\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: virtualization\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn cp_protection_exception(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: cp_protection_exception\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn hv_injection_exception(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: hv_injection_exception\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn vmm_communication_exception(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: vmm_communication_exception\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn security_exception(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: security_exception\n{:#?}", stack_frame);
}
