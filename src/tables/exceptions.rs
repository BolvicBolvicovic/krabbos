use bitflags::bitflags;
use crate::tables::selectors::SegmentSelector;
use crate::println;
use volatile::Volatile;
use core::{fmt, ops::Deref};

#[repr(transparent)]
pub struct InterruptStackFrame(InterruptStackFrameValue);

impl InterruptStackFrame {
    /// Creates a new interrupt stack frame with the given values.
    #[inline]
    pub fn new(
        instruction_pointer: u64,
        code_segment: SegmentSelector,
        cpu_flags: RFlags,
        stack_pointer: u64,
        stack_segment: SegmentSelector,
    ) -> Self {
        Self(InterruptStackFrameValue::new(
            instruction_pointer,
            code_segment,
            cpu_flags,
            stack_pointer,
            stack_segment,
        ))
    }

    /// Gives mutable access to the contents of the interrupt stack frame.
    ///
    /// The `Volatile` wrapper is used because LLVM optimizations remove non-volatile
    /// modifications of the interrupt stack frame.
    ///
    /// ## Safety
    ///
    /// This function is unsafe since modifying the content of the interrupt stack frame
    /// can easily lead to undefined behavior. For example, by writing an invalid value to
    /// the instruction pointer field, the CPU can jump to arbitrary code at the end of the
    /// interrupt.
    ///
    /// Also, it is not fully clear yet whether modifications of the interrupt stack frame are
    /// officially supported by LLVM's x86 interrupt calling convention.
    #[inline]
    pub unsafe fn as_mut(&mut self) -> Volatile<&mut InterruptStackFrameValue> {
        Volatile::new(&mut self.0)
    }
}

impl Deref for InterruptStackFrame {
    type Target = InterruptStackFrameValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for InterruptStackFrame {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    /// This value points to the instruction that should be executed when the interrupt
    /// handler returns. For most interrupts, this value points to the instruction immediately
    /// following the last executed instruction. However, for some exceptions (e.g., page faults),
    /// this value points to the faulting instruction, so that the instruction is restarted on
    /// return. See the documentation of the [`InterruptDescriptorTable`] fields for more details.
    pub instruction_pointer: u64,
    /// The code segment selector at the time of the interrupt.
    pub code_segment: SegmentSelector,
    _reserved1: [u8; 6],
    /// The flags register before the interrupt handler was invoked.
    pub cpu_flags: RFlags,
    /// The stack pointer at the time of the interrupt.
    pub stack_pointer: u64,
    /// The stack segment descriptor at the time of the interrupt (often zero in 64-bit mode).
    pub stack_segment: SegmentSelector,
    _reserved2: [u8; 6],
}

impl InterruptStackFrameValue {
    /// Creates a new interrupt stack frame with the given values.
    #[inline]
    pub fn new(
        instruction_pointer: u64,
        code_segment: SegmentSelector,
        cpu_flags: RFlags,
        stack_pointer: u64,
        stack_segment: SegmentSelector,
    ) -> Self {
        Self {
            instruction_pointer,
            code_segment,
            _reserved1: Default::default(),
            cpu_flags,
            stack_pointer,
            stack_segment,
            _reserved2: Default::default(),
        }
    }

    pub unsafe fn iretq(&self) -> ! {
        unsafe {
            core::arch::asm!(
                "push {stack_segment:r}",
                "push {new_stack_pointer}",
                "push {rflags}",
                "push {code_segment:r}",
                "push {new_instruction_pointer}",
                "iretq",
                rflags = in(reg) self.cpu_flags.bits(),
                new_instruction_pointer = in(reg) self.instruction_pointer,
                new_stack_pointer = in(reg) self.stack_pointer,
                code_segment = in(reg) self.code_segment.0,
                stack_segment = in(reg) self.stack_segment.0,
                options(noreturn)
            )
        }
    }
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("InterruptStackFrame");
        s.field("instruction_pointer", &format_args!("{:#x}", self.instruction_pointer));
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &self.cpu_flags);
        s.field("stack_pointer", &format_args!("{:#x}", self.stack_pointer));
        s.field("stack_segment", &format_args!("{:#x}", self.stack_segment.0));
        s.finish()
    }
}

bitflags! {
    /// The RFLAGS register. All bit patterns are valid representations for this type.
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct RFlags: u64 {
        /// Processor feature identification flag.
        ///
        /// If this flag is modifiable, the CPU supports CPUID.
        const ID = 1 << 21;
        /// Indicates that an external, maskable interrupt is pending.
        ///
        /// Used when virtual-8086 mode extensions (CR4.VME) or protected-mode virtual
        /// interrupts (CR4.PVI) are activated.
        const VIRTUAL_INTERRUPT_PENDING = 1 << 20;
        /// Virtual image of the INTERRUPT_FLAG bit.
        ///
        /// Used when virtual-8086 mode extensions (CR4.VME) or protected-mode virtual
        /// interrupts (CR4.PVI) are activated.
        const VIRTUAL_INTERRUPT = 1 << 19;
        /// Enable automatic alignment checking if CR0.AM is set. Only works if CPL is 3.
        const ALIGNMENT_CHECK = 1 << 18;
        /// Enable the virtual-8086 mode.
        const VIRTUAL_8086_MODE = 1 << 17;
        /// Allows to restart an instruction following an instruction breakpoint.
        const RESUME_FLAG = 1 << 16;
        /// Used by `iret` in hardware task switch mode to determine if current task is nested.
        const NESTED_TASK = 1 << 14;
        /// The high bit of the I/O Privilege Level field.
        ///
        /// Specifies the privilege level required for executing I/O address-space instructions.
        const IOPL_HIGH = 1 << 13;
        /// The low bit of the I/O Privilege Level field.
        ///
        /// Specifies the privilege level required for executing I/O address-space instructions.
        const IOPL_LOW = 1 << 12;
        /// Set by hardware to indicate that the sign bit of the result of the last signed integer
        /// operation differs from the source operands.
        const OVERFLOW_FLAG = 1 << 11;
        /// Determines the order in which strings are processed.
        const DIRECTION_FLAG = 1 << 10;
        /// Enable interrupts.
        const INTERRUPT_FLAG = 1 << 9;
        /// Enable single-step mode for debugging.
        const TRAP_FLAG = 1 << 8;
        /// Set by hardware if last arithmetic operation resulted in a negative value.
        const SIGN_FLAG = 1 << 7;
        /// Set by hardware if last arithmetic operation resulted in a zero value.
        const ZERO_FLAG = 1 << 6;
        /// Set by hardware if last arithmetic operation generated a carry ouf of bit 3 of the
        /// result.
        const AUXILIARY_CARRY_FLAG = 1 << 4;
        /// Set by hardware if last result has an even number of 1 bits (only for some operations).
        const PARITY_FLAG = 1 << 2;
        /// Set by hardware if last arithmetic operation generated a carry out of the
        /// most-significant bit of the result.
        const CARRY_FLAG = 1;
    }
}

pub fn divide_error(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

pub fn debug(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

pub fn non_maskable_interrupt(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

pub fn breakpoint(stack_frame: InterruptStackFrameValue) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub fn overflow(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

pub fn bound_range_exceeded(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

pub fn invalid_opcode(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: INVALID OP CODE\n{:#?}", stack_frame);
}

pub fn coprocessor_not_available(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: COPROCESSOR NOT AVAILABLE\n{:#?}", stack_frame);
}

pub fn double_fault(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub fn invalid_tss(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
}

pub fn segment_not_present(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: SEGMENT NOT PRESENT\n{:#?}", stack_frame);
}

pub fn stack_segment_fault(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: STACK SEGMENT FAULT\n{:#?}", stack_frame);
}

pub fn general_protection_fault(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: GPF\n{:#?}", stack_frame);
}

pub fn page_fault(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
}
pub fn x87_floating_point(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: x87_floating_point\n{:#?}", stack_frame);
}

pub fn alignment_check(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: alignment_check\n{:#?}", stack_frame);
}

pub fn machine_check(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: machine_check\n{:#?}", stack_frame);
}

pub fn simd_floating_point(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: simd_floating_point\n{:#?}", stack_frame);
}

pub fn virtualization(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: virtualization\n{:#?}", stack_frame);
}

pub fn cp_protection_exception(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: cp_protection_exception\n{:#?}", stack_frame);
}

pub fn hv_injection_exception(stack_frame: InterruptStackFrameValue) {
    panic!("EXCEPTION: hv_injection_exception\n{:#?}", stack_frame);
}

pub fn vmm_communication_exception(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: vmm_communication_exception\n{:#?}", stack_frame);
}

pub fn security_exception(stack_frame: InterruptStackFrameValue, _errcode: u64) {
    panic!("EXCEPTION: security_exception\n{:#?}", stack_frame);
}
