use crate::tables::selectors::{Segment, SegmentSelector, CS};
use crate::tables::DescriptorTablePointer;
use core::arch::asm;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = { 
        use crate::as_fn_ptr;

        let mut idt = InterruptDescriptorTable::new();
        idt.exceptions[0].set_entry(as_fn_ptr!(crate::tables::exceptions::divide_error));
        idt.exceptions[1].set_entry(as_fn_ptr!(crate::tables::exceptions::debug));
        idt.exceptions[2].set_entry(as_fn_ptr!(crate::tables::exceptions::non_maskable_interrupt));
        idt.exceptions[3].set_entry(as_fn_ptr!(crate::tables::exceptions::breakpoint));
        idt.exceptions[4].set_entry(as_fn_ptr!(crate::tables::exceptions::overflow));
        idt.exceptions[5].set_entry(as_fn_ptr!(crate::tables::exceptions::bound_range_exceeded));
        idt.exceptions[6].set_entry(as_fn_ptr!(crate::tables::exceptions::invalid_opcode));
        idt.exceptions[7].set_entry(as_fn_ptr!(crate::tables::exceptions::coprocessor_not_available));
        idt.exceptions[8].set_entry(as_fn_ptr!(crate::tables::exceptions::double_fault));
        unsafe { idt.exceptions[8].set_ist_index(0); }
        idt.exceptions[10].set_entry(as_fn_ptr!(crate::tables::exceptions::invalid_tss));
        idt.exceptions[11].set_entry(as_fn_ptr!(crate::tables::exceptions::segment_not_present));
        idt.exceptions[12].set_entry(as_fn_ptr!(crate::tables::exceptions::stack_segment_fault));
        idt.exceptions[13].set_entry(as_fn_ptr!(crate::tables::exceptions::general_protection_fault));
        idt.exceptions[14].set_entry(as_fn_ptr!(crate::tables::exceptions::page_fault));
        idt.exceptions[16].set_entry(as_fn_ptr!(crate::tables::exceptions::x87_floating_point));
        idt.exceptions[17].set_entry(as_fn_ptr!(crate::tables::exceptions::alignment_check));
        idt.exceptions[18].set_entry(as_fn_ptr!(crate::tables::exceptions::machine_check));
        idt.exceptions[19].set_entry(as_fn_ptr!(crate::tables::exceptions::simd_floating_point));
        idt.exceptions[20].set_entry(as_fn_ptr!(crate::tables::exceptions::virtualization));
        idt.exceptions[21].set_entry(as_fn_ptr!(crate::tables::exceptions::cp_protection_exception));
        idt.exceptions[28].set_entry(as_fn_ptr!(crate::tables::exceptions::hv_injection_exception));
        idt.exceptions[29].set_entry(as_fn_ptr!(crate::tables::exceptions::vmm_communication_exception));
        idt.exceptions[30].set_entry(as_fn_ptr!(crate::tables::exceptions::security_exception));
        idt
    };
}

pub fn load_idt() {
    IDT.load();
}

#[repr(C)]
pub struct InterruptDescriptorTable {
    pub exceptions: [IDTEntry; 32],
    pub interrupts: [IDTEntry; 224],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        InterruptDescriptorTable {
            exceptions: [IDTEntry::missing(); 32],
            interrupts: [IDTEntry::missing(); 224],
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn load(&'static self) {
        unsafe {
            let ptr = self.pointer();
            asm!("lidt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags))
        }
    }


    fn pointer(&self) -> DescriptorTablePointer {
        use core::mem::size_of;
        DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IDTEntry {
    pointer_low:    u16,
    cs:             SegmentSelector,
    options:        u16,
    pointer_mid:    u16,
    pointer_high:   u32,
    reserved:       u32,
}

impl IDTEntry {

    #[inline]
    pub const fn missing() -> Self {
        IDTEntry {
            pointer_low: 0,
            pointer_mid: 0,
            pointer_high:0,
            cs:          SegmentSelector(0),
            options:     0b1110_0000_0000,
            reserved:    0,
        }
    }

    pub fn set_entry(&mut self, addr: u64) {
        self.pointer_low = addr as u16;
        self.pointer_mid = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;
        self.cs = CS::get_reg();
        self.set_present(true);
    }

    #[inline]
    pub fn set_present(&mut self, present: bool) {
        if present {
            self.options |= 0b1000_0000_0000_0000u16; // bit nb 15
        } else {
            self.options &= !0b1000_0000_0000_0000u16; // bit nb 15
        }
    }

    fn present(&self) -> bool {
        self.options & 0b1000_0000_0000_0000u16 != 0
    }

    #[inline]
    pub fn disable_interrupts(&mut self, disable: bool) {
        if disable {
            self.options &= !0b1000_0000u16; // bit nb 8
        } else {
            self.options |= 0b1000_0000u16;
        }
    }

    #[inline]
    pub fn set_privilege_level(&mut self, dpl: u16) {
        if dpl > 3 { panic!("Panic setting dpl for IDTEntry") }
        self.options &= !0b110_0000_0000_0000u16; // bits nb 13, 14
        self.options |= dpl << 13;
    }

    #[inline]
    pub unsafe fn set_ist_index(&mut self, index: u16) {
        if index > 2 { panic!("Panic setting IST index for IDTEntry") }
        self.options &= !0b11u16; // bits nb 0, 1
        self.options |= index;
    }

    fn stack_index(&self) -> u16 {
        self.options & 0b11u16 - 1
    }
}
