use alloc::vec::Vec;
use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

// очередь сканкодов
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

// состояние строки
static mut LINE_BUFFER: [u8; 256] = [0; 256];
static mut BUFFER_POS: usize = 0;

// макс. размер строки
const BUFFER_CAPACITY: usize = 256;

pub async fn run_shell() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore
    );

    print!("> ");

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        match character {
                            '\n' | '\r' => {
                                println!();
                                process_command().await;
                                print!("> ");
                            }
                            '\x08' | '\x7F' => { // backspace или delete
                                unsafe {
                                    let pos_ptr: *const usize = &raw const BUFFER_POS;
                                    if *pos_ptr > 0 {
                                        let pos_mut: *mut usize = &raw mut BUFFER_POS;
                                        *pos_mut -= 1;
                                        print!("\x08 \x08");
                                    }
                                }
                            }
                            _ => {
                                print!("{}", character);
                                unsafe {
                                    let pos_ptr: *const usize = &raw const BUFFER_POS;
                                    let current_pos = *pos_ptr;
                                    
                                    if current_pos < BUFFER_CAPACITY {
                                        let buf_ptr = (&raw mut LINE_BUFFER).cast::<u8>();
                                        *buf_ptr.add(current_pos) = character as u8;
                                        
                                        let pos_mut: *mut usize = &raw mut BUFFER_POS;
                                        *pos_mut = current_pos + 1;
                                    }
                                }
                            }
                        }
                    }
                    DecodedKey::RawKey(_key) => {}
                }
            }
        }
    }
}

async fn process_command() {
    // получение строки из буфера
    let cmd = unsafe {
        let pos = *(&raw const BUFFER_POS);
        if pos == 0 {
            ""
        } else {
            let buf_ptr = &raw const LINE_BUFFER;
            let slice = core::slice::from_raw_parts(buf_ptr.cast::<u8>(), pos);
            core::str::from_utf8(slice).unwrap_or("").trim()
        }
    };
    
    // очистка буфера
    unsafe {
        let pos_mut: *mut usize = &raw mut BUFFER_POS;
        *pos_mut = 0;
    }

    if cmd.is_empty() {
        return;
    }

    // парсинг команд
    let mut parts = cmd.split_whitespace();
    let command = parts.next().unwrap_or("");
    let args = parts.collect::<Vec<&str>>().join(" ");

    match command {
        "echo" => {
            println!("{}", args);
        }
        "help" => {
            println!("Available commands: echo, help");
        }
        "clear" => {
            // костыль ебучий
            for _ in 0..25 {
                println!();
            }
        }
        _ => {
            println!("Unknown command: '{}'", command);
        }
    }
}
