use core::fmt;
use bootloader_api::info::{PixelFormat, FrameBuffer};
use crate::font::FONT;

const CHAR_W: usize = 8;
const CHAR_H: usize = 16;
const COLS: usize = 80;
const ROWS: usize = 25;

#[derive(Debug)]
pub struct FrameBufferWriter {
    buffer: &'static mut [u8],
    info: bootloader_api::info::FrameBufferInfo,
    cursor_x: usize,
    cursor_y: usize,
    fg: u32,
    bg: u32,
}

impl FrameBufferWriter {
    fn new(buffer: &'static mut [u8], info: bootloader_api::info::FrameBufferInfo) -> Self {
        Self {
            buffer,
            info,
            cursor_x: 0,
            cursor_y: 0,
            fg: 0x00FFFFFF,
            bg: 0x00000000,
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.info.width as usize || y >= self.info.height as usize {
            return;
        }
        let offset = (y * self.info.stride as usize + x) * 4;
        let bytes = match self.info.pixel_format {
            PixelFormat::Bgr => color.to_le_bytes(),
            PixelFormat::Rgb => color.to_be_bytes(),
            _ => color.to_le_bytes(),
        };
        if offset + 4 <= self.buffer.len() {
            self.buffer[offset..offset + 4].copy_from_slice(&bytes);
        }
    }

    fn draw_char(&mut self, x: usize, y: usize, ch: u8) {
        if ch < 32 || ch > 126 { return; }
        let idx = (ch - 32) as usize;
        let base_x = x * CHAR_W;
        let base_y = y * CHAR_H;

        for row in 0..CHAR_H {
            let line = FONT[idx * CHAR_H + row];
            for col in 0..CHAR_W {
                let bit = (line >> (7 - col)) & 1;
                let color = if bit != 0 { self.fg } else { self.bg };
                self.write_pixel(base_x + col, base_y + row, color);
            }
        }
    }

    fn scroll(&mut self) {
        let bytes_per_line = self.info.stride as usize * CHAR_H * 4;
        let total_height = self.info.height as usize;
        
        for y in CHAR_H..total_height {
            let src = y * self.info.stride as usize * 4;
            let dst = (y - CHAR_H) * self.info.stride as usize * 4;
            self.buffer.copy_within(src..src + bytes_per_line, dst);
        }

        for x in 0..self.info.width as usize {
            for y in (total_height - CHAR_H)..total_height {
                self.write_pixel(x, y, self.bg);
            }
        }
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }

    fn new_line(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= ROWS {
            self.scroll();
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => { self.cursor_x = 0; }
            b'\x08' => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    self.draw_char(self.cursor_x, self.cursor_y, b' ');
                }
            }
            _ => {
                if self.cursor_x >= COLS {
                    self.new_line();
                }
                self.draw_char(self.cursor_x, self.cursor_y, byte);
                self.cursor_x += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7E | b'\n' | b'\r' | b'\x08' => self.write_byte(byte),
                _ => self.write_byte(b'?'),
            }
        }
    }
}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

static WRITER: spin::Mutex<Option<FrameBufferWriter>> = spin::Mutex::new(None);

pub fn init(framebuffer: Option<FrameBuffer>) {
    if let Some(fb) = framebuffer {
        let info = fb.info();
        let buffer = fb.into_buffer();
        let mut w = FrameBufferWriter::new(buffer, info);
        
        for y in 0..info.height as usize {
            for x in 0..info.width as usize {
                w.write_pixel(x, y, 0x00000000);
            }
        }
        *WRITER.lock() = Some(w);
    }
}
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(writer) = &mut *WRITER.lock() {
        let _ = writer.write_fmt(args);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
