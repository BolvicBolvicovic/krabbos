use core::arch::asm;

#[repr(C)]
pub struct Port(u16);


impl Port {
    pub const fn new(p: u16) -> Self {
        Self(p)
    }

    pub unsafe fn write<T: PortWrite>(&self, value: T) {
        unsafe { value.write_to_port(self.0); };
    }

    pub unsafe fn read<T: PortRead>(&self, value: T) -> T {
        unsafe { value.read_from_port(self.0) }
    }
}

pub trait PortWrite {
    unsafe fn write_to_port(self, port: u16);
}

impl PortWrite for u8 {
    unsafe fn write_to_port(self, port: u16) {
        unsafe {
            asm!("out dx, al", in("dx") port, in("al") self, options(nomem, nostack, preserves_flags));
        }
    }
}

impl PortWrite for u16 {
    unsafe fn write_to_port(self, port: u16) {
        unsafe {
            asm!("out dx, al", in("dx") port, in("ax") self, options(nomem, nostack, preserves_flags));
        }
    }
}

impl PortWrite for u32 {
    unsafe fn write_to_port(self, port: u16) {
        unsafe {
            asm!("out dx, eax", in("dx") port, in("eax") self, options(nomem, nostack, preserves_flags));
        }
    }
}

pub trait PortRead {
    unsafe fn read_from_port(self, port: u16) -> Self;
}

impl PortRead for u8 {
    unsafe fn read_from_port(self, port: u16) -> Self {
        let value: u8;
        unsafe {
            asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
        }
        value
    }
}

impl PortRead for u16 {
    unsafe fn read_from_port(self, port: u16) -> Self {
        let value: u16;
        unsafe {
            asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        }
        value
    }
}

impl PortRead for u32 {
    unsafe fn read_from_port(self, port: u16) -> Self {
        let value: u32;
        unsafe {
            asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        }
        value
    }
}
