#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]

mod vga;
mod tables;
mod pic;

use core::{panic::PanicInfo, arch::asm};
use pic::timer::init_pit;
use tables::{idt::load_idt, port::Port, gdt::load_gdt};

#[no_mangle] // That forces the compiler to keep the name of the function as it is.
pub extern "C" fn _start() -> ! {
    println!("Hello, World from krabbos!");

    load_gdt();
    load_idt();
    unsafe { 
        pic::PICS.lock().initialize();
        init_pit(50);

        // Sets interrupts
        asm!( "sti", options(preserves_flags, nostack) );
    };


    #[cfg(test)]
    test_main();

    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}


#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    print!("This is a trivial assertion to test the test custom framework... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
