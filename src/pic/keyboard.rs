use crate::{pic::PICS, tables::{port::Port, InterruptStackFrame}, print};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;

const SCANCODE_PORT: u16 = 0x60;

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet1>> =
            Mutex::new(Keyboard::new(ScancodeSet1::new(),
                layouts::Azerty, HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let port = Port::new(SCANCODE_PORT);

    let mut scancode: u8 = 0;
    scancode = unsafe { port.read(scancode) };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(_key) => {},
            }
        }
    }

    unsafe { PICS.lock().notify_end_of_interrupt(33); }
}
