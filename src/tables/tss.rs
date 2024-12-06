use core::{mem::size_of, ptr::addr_of};
use lazy_static::lazy_static;
use core::arch::asm;

use super::selectors::SegmentSelector;

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0 as usize] = {
            const STACK_SIZE: u64 = 0x1000 * 5;
            static mut STACK: [u8; STACK_SIZE as usize] = [0; STACK_SIZE as usize];
            let stack_start = addr_of!(STACK) as u64;
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}


/// In 64-bit mode the TSS holds information that is not
/// directly related to the task-switch mechanism,
/// but is used for stack switching when an interrupt or exception occurs.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,
    /// The full 64-bit canonical forms of the stack pointers (RSP) for privilege levels 0-2.
    /// The stack pointers used when a privilege level change occurs from a lower privilege level to a higher one.
    pub privilege_stack_table: [u64; 3],
    reserved_2: u64,
    /// The full 64-bit canonical forms of the interrupt stack table (IST) pointers.
    /// The stack pointers used when an entry in the Interrupt Descriptor Table has an IST value other than 0.
    pub interrupt_stack_table: [u64; 7],
    reserved_3: u64,
    reserved_4: u16,
    /// The 16-bit offset to the I/O permission bit map from the 64-bit TSS base.
    pub iomap_base: u16,
}

impl TaskStateSegment {
    /// Creates a new TSS with zeroed privilege and interrupt stack table and an
    /// empty I/O-Permission Bitmap.
    ///
    /// As we always set the TSS segment limit to
    /// `size_of::<TaskStateSegment>() - 1`, this means that `iomap_base` is
    /// initialized to `size_of::<TaskStateSegment>()`.
    #[inline]
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            iomap_base: size_of::<TaskStateSegment>() as u16,
            reserved_1: 0,
            reserved_2: 0,
            reserved_3: 0,
            reserved_4: 0,
        }
    }

    pub unsafe fn load(&self, ss: SegmentSelector) {
        unsafe {
            asm!("ltr {0:x}", in(reg) ss.0, options(nostack, preserves_flags));
        }
    }
}

impl Default for TaskStateSegment {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
