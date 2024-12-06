pub mod idt;
pub mod port;
pub mod selectors;
pub mod gdt;
mod exceptions;
mod tss;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: u64,
}

#[macro_export]
macro_rules! as_fn_ptr {
    ($($arg:tt)*) => { ($($arg)* as *const () as u64) }
}
