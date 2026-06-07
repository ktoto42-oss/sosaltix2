use alloc::vec::Vec;
use crate::{print, println, serial_print};
use crate::task::SimpleQueue;
use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
};
use spin::Mutex;

static SCANCODE_QUEUE: Mutex<SimpleQueue<u8, 100>> = Mutex::new(SimpleQueue::new());
static WAKER: Mutex<Option<Waker>> = Mutex::new(None);

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        ScancodeStream { _private: () }
    }

    pub fn next(&mut self) -> ScancodeNext<'_> {
        ScancodeNext { _stream: self }
    }
}

pub struct ScancodeNext<'a> {
    _stream: &'a mut ScancodeStream,
}

impl<'a> core::future::Future for ScancodeNext<'a> {
    type Output = Option<u8>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut queue = SCANCODE_QUEUE.lock();

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        *WAKER.lock() = Some(cx.waker().clone());

        match queue.pop() {
            Some(scancode) => {
                *WAKER.lock() = None;
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub fn add_scancode(scancode: u8) {
    let mut queue = SCANCODE_QUEUE.lock();
    if queue.push(scancode).is_err() {
        println!("WARNING: scancode queue full; dropping keyboard input");
    } else {
        if let Some(waker) = WAKER.lock().take() {
            waker.wake();
        }
    }
}

pub enum Key {
    Char(char),
    ArrowLeft,
    ArrowRight,
}

pub fn map_scancode(scancode: u8, extended: bool) -> Option<Key> {
    if extended {
        return match scancode {
            0x4B => Some(Key::ArrowLeft),
            0x4D => Some(Key::ArrowRight),
            _ => None,
        };
    }

    if scancode >= 0x80 {
        return None;
    }

    match scancode {
        0x0E => Some(Key::Char('\x08')),
        0x1C => Some(Key::Char('\n')),
        0x39 => Some(Key::Char(' ')),
        
        0x02 => Some(Key::Char('1')), 0x03 => Some(Key::Char('2')),
        0x04 => Some(Key::Char('3')), 0x05 => Some(Key::Char('4')),
        0x06 => Some(Key::Char('5')), 0x07 => Some(Key::Char('6')),
        0x08 => Some(Key::Char('7')), 0x09 => Some(Key::Char('8')),
        0x0A => Some(Key::Char('9')), 0x0B => Some(Key::Char('0')),
        0x0C => Some(Key::Char('-')), 0x0D => Some(Key::Char('=')),
        
        0x10 => Some(Key::Char('q')), 0x11 => Some(Key::Char('w')),
        0x12 => Some(Key::Char('e')), 0x13 => Some(Key::Char('r')),
        0x14 => Some(Key::Char('t')), 0x15 => Some(Key::Char('y')),
        0x16 => Some(Key::Char('u')), 0x17 => Some(Key::Char('i')),
        0x18 => Some(Key::Char('o')), 0x19 => Some(Key::Char('p')),
        0x1A => Some(Key::Char('[')), 0x1B => Some(Key::Char(']')),
        
        0x1E => Some(Key::Char('a')), 0x1F => Some(Key::Char('s')),
        0x20 => Some(Key::Char('d')), 0x21 => Some(Key::Char('f')),
        0x22 => Some(Key::Char('g')), 0x23 => Some(Key::Char('h')),
        0x24 => Some(Key::Char('j')), 0x25 => Some(Key::Char('k')),
        0x26 => Some(Key::Char('l')), 0x27 => Some(Key::Char(';')),
        0x28 => Some(Key::Char('\'')), 0x29 => Some(Key::Char('`')),
        0x2B => Some(Key::Char('\\')),
        
        0x2C => Some(Key::Char('z')), 0x2D => Some(Key::Char('x')),
        0x2E => Some(Key::Char('c')), 0x2F => Some(Key::Char('v')),
        0x30 => Some(Key::Char('b')), 0x31 => Some(Key::Char('n')),
        0x32 => Some(Key::Char('m')), 0x33 => Some(Key::Char(',')),
        0x34 => Some(Key::Char('.')), 0x35 => Some(Key::Char('/')),
        
        _ => None,
    }
}

static mut LINE_BUFFER: [u8; 256] = [0; 256];
static mut BUFFER_LEN: usize = 0;
static mut CURSOR_POS: usize = 0;
const BUFFER_CAPACITY: usize = 256;

pub async fn run_shell() {
    let mut scancodes = ScancodeStream::new();
    let mut extended = false;

    print!("> ");

    while let Some(scancode) = scancodes.next().await {
        if scancode == 0xE0 {
            extended = true;
            continue;
        }

        let key = map_scancode(scancode, extended);
        extended = false;

        if let Some(key) = key {
            match key {
                Key::Char(character) => {
                    match character {
                        '\n' | '\r' => {
                            println!();
                            process_command().await;
                            print!("> ");
                        }
                        '\x08' | '\x7F' => { // backspace
                            unsafe {
                                if CURSOR_POS > 0 {
                                    let idx = CURSOR_POS - 1;
                                    for i in idx..(BUFFER_LEN - 1) {
                                        LINE_BUFFER[i] = LINE_BUFFER[i + 1];
                                    }
                                    BUFFER_LEN -= 1;
                                    CURSOR_POS -= 1;

                                    print!("\x08");

                                    let buf_ptr = &raw const LINE_BUFFER;
                                    let slice = core::slice::from_raw_parts(buf_ptr.cast::<u8>(), BUFFER_LEN);
                                    if let Ok(tail) = core::str::from_utf8(&slice[CURSOR_POS..]) {
                                        print!("{}", tail);
                                    }

                                    print!(" \x08");

                                    let shift_back = BUFFER_LEN - CURSOR_POS;
                                    if shift_back > 0 {
                                        print!("\x1B[{}D", shift_back);
                                    }
                                }
                            }
                        }
                        _ => {
                            print!("{}", character);
                            unsafe {
                                if CURSOR_POS < BUFFER_CAPACITY {
                                    LINE_BUFFER[CURSOR_POS] = character as u8;
                                    CURSOR_POS += 1;
                                    
                                    if CURSOR_POS > BUFFER_LEN {
                                        BUFFER_LEN = CURSOR_POS;
                                    }
                                }
                            }
                        }
                    }
                }
                Key::ArrowLeft => {
                    unsafe {
                        if CURSOR_POS > 0 {
                            CURSOR_POS -= 1;
                            print!("\x1B[D");
                            crate::terminal::redraw();
                        }
                    }
                }
                Key::ArrowRight => {
                    unsafe {
                        if CURSOR_POS < BUFFER_LEN {
                            CURSOR_POS += 1;
                            print!("\x1B[C");
                            crate::terminal::redraw();
                        }
                    }
                }
            }
        }
    }
}

async fn process_command() {
    let cmd = unsafe {
        if BUFFER_LEN == 0 {
            ""
        } else {
            let buf_ptr = &raw const LINE_BUFFER;
            let slice = core::slice::from_raw_parts(buf_ptr.cast::<u8>(), BUFFER_LEN);
            core::str::from_utf8(slice).unwrap_or("").trim()
        }
    };
    
    unsafe {
        BUFFER_LEN = 0;
        CURSOR_POS = 0;
    }

    if cmd.is_empty() {
        return;
    }

    let mut parts = cmd.split_whitespace();
    let command = parts.next().unwrap_or("");
    let args = parts.collect::<Vec<&str>>().join(" ");

    match command {
        "echo" => { crate::commands::base::echo(args); }
        "help" => { crate::commands::base::help(); }
        "clear" => { crate::terminal::clear_screen(); }
        "fetch" => { crate::commands::fetch::fetch(); }
        "reboot" => { crate::commands::power::reboot(); }
        "poweroff" => { crate::commands::power::poweroff(); }
        "disk-status" => { crate::commands::base::print_disk_status(); }
        "read-sector" => { crate::commands::base::read_sector_cmd(0); }
        _ => {
            println!("Unknown command: '{}'", command);
        }
    }
}