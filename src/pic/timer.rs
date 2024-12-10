use crate::{pic::PICS, tables::{port::Port, InterruptStackFrame}};

const PIT_CTRL_WORD: u16 = 0x43;
const PIT_COUNTER_0: u16 = 0x40;
const CLOCK_RATE: u64 = 1193180;

pub extern "x86-interrupt" fn pit_handler(_stack_frame: InterruptStackFrame) {
    unsafe { PICS.lock().notify_end_of_interrupt(32); }
}

pub fn init_pit(frequency: u64) {
    let divisor = CLOCK_RATE / frequency;
    let port = Port::new(PIT_CTRL_WORD);
	//    00                 11                      011                         0
	// Counter 0 | RD or LD LSB then MSB | Mode 3: Square Wave Generator | Binary counter
    unsafe { port.write(0b110110u8); }
    let port = Port::new(PIT_COUNTER_0);
    let lsb: u8 = (divisor & 0xFF) as u8;
    let msb: u8 = ((divisor >> 8) &0xFF) as u8;
    unsafe {
        port.write(lsb);
        port.write(msb);
    }
}
