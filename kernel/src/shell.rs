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
    ShiftPressed,
    ShiftReleased,
}

pub fn map_scancode(scancode: u8, extended: bool, shift: bool) -> Option<Key> {
    if extended {
        return match scancode {
            0x4B => Some(Key::ArrowLeft),
            0x4D => Some(Key::ArrowRight),
            _ => None,
        };
    }

    match scancode {
        0x2A | 0x36 => return Some(Key::ShiftPressed),
        0xAA | 0xB6 => return Some(Key::ShiftReleased),
        _ => {}
    }

    if scancode >= 0x80 {
        return None;
    }

    match scancode {
        0x0E => Some(Key::Char('\x08')),
        0x1C => Some(Key::Char('\n')),
        0x39 => Some(Key::Char(' ')),
        
        0x02 => Some(Key::Char(if shift { '!' } else { '1' })),
        0x03 => Some(Key::Char(if shift { '@' } else { '2' })),
        0x04 => Some(Key::Char(if shift { '#' } else { '3' })),
        0x05 => Some(Key::Char(if shift { '$' } else { '4' })),
        0x06 => Some(Key::Char(if shift { '%' } else { '5' })),
        0x07 => Some(Key::Char(if shift { '^' } else { '6' })),
        0x08 => Some(Key::Char(if shift { '&' } else { '7' })),
        0x09 => Some(Key::Char(if shift { '*' } else { '8' })),
        0x0A => Some(Key::Char(if shift { '(' } else { '9' })),
        0x0B => Some(Key::Char(if shift { ')' } else { '0' })),
        0x0C => Some(Key::Char(if shift { '_' } else { '-' })),
        0x0D => Some(Key::Char(if shift { '+' } else { '=' })),
        
        0x10 => Some(Key::Char(if shift { 'Q' } else { 'q' })),
        0x11 => Some(Key::Char(if shift { 'W' } else { 'w' })),
        0x12 => Some(Key::Char(if shift { 'E' } else { 'e' })),
        0x13 => Some(Key::Char(if shift { 'R' } else { 'r' })),
        0x14 => Some(Key::Char(if shift { 'T' } else { 't' })),
        0x15 => Some(Key::Char(if shift { 'Y' } else { 'y' })),
        0x16 => Some(Key::Char(if shift { 'U' } else { 'u' })),
        0x17 => Some(Key::Char(if shift { 'I' } else { 'i' })),
        0x18 => Some(Key::Char(if shift { 'O' } else { 'o' })),
        0x19 => Some(Key::Char(if shift { 'P' } else { 'p' })),
        0x1A => Some(Key::Char(if shift { '{' } else { '[' })),
        0x1B => Some(Key::Char(if shift { '}' } else { ']' })),
        
        0x1E => Some(Key::Char(if shift { 'A' } else { 'a' })),
        0x1F => Some(Key::Char(if shift { 'S' } else { 's' })),
        0x20 => Some(Key::Char(if shift { 'D' } else { 'd' })),
        0x21 => Some(Key::Char(if shift { 'F' } else { 'f' })),
        0x22 => Some(Key::Char(if shift { 'G' } else { 'g' })),
        0x23 => Some(Key::Char(if shift { 'H' } else { 'h' })),
        0x24 => Some(Key::Char(if shift { 'J' } else { 'j' })),
        0x25 => Some(Key::Char(if shift { 'K' } else { 'k' })),
        0x26 => Some(Key::Char(if shift { 'L' } else { 'l' })),
        0x27 => Some(Key::Char(if shift { ':' } else { ';' })),
        0x28 => Some(Key::Char(if shift { '"' } else { '\'' })),
        0x29 => Some(Key::Char(if shift { '~' } else { '`' })),
        0x2B => Some(Key::Char(if shift { '|' } else { '\\' })),
        
        0x2C => Some(Key::Char(if shift { 'Z' } else { 'z' })),
        0x2D => Some(Key::Char(if shift { 'X' } else { 'x' })),
        0x2E => Some(Key::Char(if shift { 'C' } else { 'c' })),
        0x2F => Some(Key::Char(if shift { 'V' } else { 'v' })),
        0x30 => Some(Key::Char(if shift { 'B' } else { 'b' })),
        0x31 => Some(Key::Char(if shift { 'N' } else { 'n' })),
        0x32 => Some(Key::Char(if shift { 'M' } else { 'm' })),
        0x33 => Some(Key::Char(if shift { '<' } else { ',' })), 
        0x34 => Some(Key::Char(if shift { '>' } else { '.' })), 
        0x35 => Some(Key::Char(if shift { '?' } else { '/' })),
        
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
    let mut shift = false;

    print!("> ");

    while let Some(scancode) = scancodes.next().await {
        if scancode == 0xE0 {
            extended = true;
            continue;
        }

        let key = map_scancode(scancode, extended, shift);
        extended = false;

        if let Some(key) = key {
            match key {
                Key::ShiftPressed => {
                    shift = true;
                }
                Key::ShiftReleased => {
                    shift = false;
                }
                Key::Char(character) => {
                    match character {
                        '\n' | '\r' => {
                            println!();
                            process_command().await;
                            print!("> ");
                        }
                        '\x08' | '\x7F' => {
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
        "read-sector" => { crate::commands::base::read_sector_cmd(args); }
        "ls" => { crate::commands::base::ls_cmd(args); }
        "cat" => { crate::commands::base::cat_cmd(args); }
        "touch" => { crate::commands::base::touch_cmd(args); }
        "mkdir" => { crate::commands::base::mkdir_cmd(args); }
        "rm" => { crate::commands::base::rm_cmd(args); }
        "read-sector" => { crate::commands::base::read_sector_cmd(args); }
        "cd" => { crate::commands::base::cd_cmd(args); }
        _ => {
            println!("Unknown command: '{}'", command);
        }
    }
}