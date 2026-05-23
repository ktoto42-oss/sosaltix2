use alloc::vec::Vec;
use crate::{print, println, serial_print};
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
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyCode};

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
static mut BUFFER_LEN: usize = 0;
static mut CURSOR_POS: usize = 0;

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
                            '\x08' | '\x7F' => { // backspace
                                unsafe {
                                    if CURSOR_POS > 0 {
                                        CURSOR_POS -= 1;
                                        // в полноценном readline здесь нужно сдвигать весь массив влево но мне лень
                                        print!("\x08 \x08");
                                        
                                        // если стирает с конца уменьшает длину буфера
                                        if CURSOR_POS == BUFFER_LEN - 1 {
                                            BUFFER_LEN -= 1;
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
                                        
                                        // если пишет за пределами старой длины увеличиваем длину
                                        if CURSOR_POS > BUFFER_LEN {
                                            BUFFER_LEN = CURSOR_POS;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    DecodedKey::RawKey(raw_key) => {
                        match raw_key {
                            KeyCode::ArrowLeft => {
                                unsafe {
                                    if CURSOR_POS > 0 {
                                        CURSOR_POS -= 1;
                                        print!("\x1B[D"); // отправляет ansi код терминалу (влево)
                                    }
                                }
                            }
                            KeyCode::ArrowRight => {
                                unsafe {
                                    if CURSOR_POS < BUFFER_LEN {
                                        CURSOR_POS += 1;
                                        print!("\x1B[C"); // отправляет ansi код терминалу (вправо)
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

async fn process_command() {
    // получение строки из буфера
    let cmd = unsafe {
        if BUFFER_LEN == 0 {
            ""
        } else {
            let buf_ptr = &raw const LINE_BUFFER;
            let slice = core::slice::from_raw_parts(buf_ptr.cast::<u8>(), BUFFER_LEN);
            core::str::from_utf8(slice).unwrap_or("").trim()
        }
    };
    
    // очистка буфера
    unsafe {
        BUFFER_LEN = 0;
        CURSOR_POS = 0;
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
            println!("{args}");
        }
        "help" => {
            println!("Available commands: echo, help, clear")
        }
        "clear" => {
            crate::terminal::clear_screen();
        }
        _ => {
            println!("Unknown command: '{}'", command);
        }
    }
}
