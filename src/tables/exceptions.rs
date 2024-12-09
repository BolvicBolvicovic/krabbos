use crate::{println, tables::InterruptStackFrame};

pub fn divide_error(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

pub fn debug(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

pub fn non_maskable_interrupt(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

pub fn breakpoint(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub fn overflow(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

pub fn bound_range_exceeded(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

pub fn invalid_opcode(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OP CODE\n{:#?}", stack_frame);
}

pub fn coprocessor_not_available(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: COPROCESSOR NOT AVAILABLE\n{:#?}", stack_frame);
}

pub fn double_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub fn invalid_tss(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
}

pub fn segment_not_present(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
}

pub fn stack_segment_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
}

pub fn general_protection_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: GPF\n{:#?}", stack_frame);
}

pub fn page_fault(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
}
pub fn x87_floating_point(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: x87_floating_point\n{:#?}", stack_frame);
}

pub fn alignment_check(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: alignment_check\n{:#?}", stack_frame);
}

pub fn machine_check(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: machine_check\n{:#?}", stack_frame);
}

pub fn simd_floating_point(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: simd_floating_point\n{:#?}", stack_frame);
}

pub fn virtualization(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: virtualization\n{:#?}", stack_frame);
}

pub fn cp_protection_exception(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: cp_protection_exception\n{:#?}", stack_frame);
}

pub fn hv_injection_exception(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: hv_injection_exception\n{:#?}", stack_frame);
}

pub fn vmm_communication_exception(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: vmm_communication_exception\n{:#?}", stack_frame);
}

pub fn security_exception(stack_frame: InterruptStackFrame, _errcode: u64) {
    panic!("EXCEPTION: security_exception\n{:#?}", stack_frame);
}
