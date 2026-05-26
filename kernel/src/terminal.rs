use core::fmt;
use bootloader_api::info::{PixelFormat, FrameBuffer, FrameBufferInfo};
use crate::font::FONT;

const CHAR_W: usize = 8;
const CHAR_H: usize = 16;

// цвета в формате rgb
const COLORS: [u32; 16] = [
    0x000000, 0xAA0000, 0x00AA00, 0xAA5500, 0x0000AA, 0xAA00AA, 0x00AAAA, 0xAAAAAA, // Dark
    0x555555, 0xFF5555, 0x55FF55, 0xFFFF55, 0x5555FF, 0xFF55FF, 0x55FFFF, 0xFFFFFF, // Bright
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnsiState {
    Normal,
    Escaped,
    ParsingCsi,
}

pub struct Terminal {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    cols: usize,
    rows: usize,
    cursor_x: usize,
    cursor_y: usize,
    fg_color: u32,
    bg_color: u32,
    
    // для парсинга ansi
    ansi_state: AnsiState,
    ansi_param: usize,
}

impl Terminal {
    pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let cols = info.width as usize / CHAR_W;
        let rows = info.height as usize / CHAR_H;
        
        let mut term = Self {
            buffer,
            info,
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            fg_color: COLORS[7], // светло серый
            bg_color: COLORS[0], // чёрный
            ansi_state: AnsiState::Normal,
            ansi_param: 0,
        };
        term.clear_screen();
        term
    }

    fn write_pixel(&mut self, x: usize, y: usize, color_rgb: u32) {
        if x >= self.info.width as usize || y >= self.info.height as usize { return; }
        let offset = (y * self.info.stride as usize + x) * 4;
        
        // конвертация rgb цвета в цвет фреймбуффера
        let bytes = match self.info.pixel_format {
            PixelFormat::Rgb => color_rgb.to_be_bytes(), // 0x00RRGGBB > [00, RR, GG, BB]
            PixelFormat::Bgr => {
                let r = (color_rgb >> 16) & 0xFF;
                let g = (color_rgb >> 8) & 0xFF;
                let b = color_rgb & 0xFF;
                (b << 16 | g << 8 | r).to_le_bytes() // меняет местами для bgr
            },
            PixelFormat::U8 => {
                // вибкод
                let r = (color_rgb >> 16) & 0xFF;
                ((r as u32) * 8 / 255).to_le_bytes()
            }
            _ => color_rgb.to_le_bytes(),
        };

        if offset + 4 <= self.buffer.len() {
            self.buffer[offset..offset + 4].copy_from_slice(&bytes);
        }
    }

    pub fn clear_screen(&mut self) {
        for y in 0..self.info.height as usize {
            for x in 0..self.info.width as usize {
                self.write_pixel(x, y, self.bg_color);
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
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
                let color = if bit != 0 { self.fg_color } else { self.bg_color };
                self.write_pixel(base_x + col, base_y + row, color);
            }
        }
    }

    fn scroll(&mut self) {
        let row_bytes = self.info.stride as usize * 4;
        let total_height = self.info.height as usize;
    
        // смещение 
        let text_row_offset = CHAR_H * row_bytes;
        let buffer_end = total_height * row_bytes;

        // сдвигает память фреймбуффера
        if buffer_end > text_row_offset {
            self.buffer.copy_within(text_row_offset..buffer_end, 0);
        }

        // очищает последнюю строку (куда перейдет курсор)
        for x in 0..self.info.width as usize {
            for y in (total_height - CHAR_H)..total_height {
                self.write_pixel(x, y, self.bg_color);
            }
        }
    
        // поднимает курсор на строку вверх
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }

    fn new_line(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.scroll();
        }
    }

    // обработка одного байта 
    pub fn write_byte(&mut self, byte: u8) {
        match self.ansi_state {
            AnsiState::Normal => {
                match byte {
                    0x1B => self.ansi_state = AnsiState::Escaped, // начало ansi (esc)
                    b'\n' => self.new_line(),
                    b'\r' => self.cursor_x = 0,
                    b'\x08' => { // backspace
                        if self.cursor_x > 0 {
                            self.cursor_x -= 1;
                            self.draw_char(self.cursor_x, self.cursor_y, b' ');
                        }
                    }
                    0x20..=0x7E => { // обычные символы
                        if self.cursor_x >= self.cols {
                            self.new_line();
                        }
                        self.draw_char(self.cursor_x, self.cursor_y, byte);
                        self.cursor_x += 1;
                    }
                    _ => {} 
                }
            }
            AnsiState::Escaped => {
                match byte {
                    b'[' => {
                        self.ansi_state = AnsiState::ParsingCsi;
                        self.ansi_param = 0;
                    }
                    _ => self.ansi_state = AnsiState::Normal, // сброс при ошибке
                }
            }
            AnsiState::ParsingCsi => {
                match byte {
                    b'0'..=b'9' => {
                        self.ansi_param = self.ansi_param * 10 + (byte - b'0') as usize;
                    }
                    b'm' => { // команда изменения цвета
                        match self.ansi_param {
                            0 => { // сброс
                                self.fg_color = COLORS[7];
                                self.bg_color = COLORS[0];
                            }
                            30..=37 => self.fg_color = COLORS[self.ansi_param - 30], // текст
                            90..=97 => self.fg_color = COLORS[self.ansi_param - 90 + 8], // яркий текст
                            40..=47 => self.bg_color = COLORS[self.ansi_param - 40], // фон
                            100..=107 => self.bg_color = COLORS[self.ansi_param - 100 + 8], // яркий фон
                            _ => {}
                        }
                        self.ansi_state = AnsiState::Normal;
                    }

                    b'C' => { // вправо
                        let steps = if self.ansi_param == 0 { 1 } else { self.ansi_param };
                        self.cursor_x = (self.cursor_x + steps).min(self.cols - 1);
                        self.ansi_state = AnsiState::Normal;
                    }
                    b'D' => { // влево
                        let steps = if self.ansi_param == 0 { 1 } else { self.ansi_param };
                        self.cursor_x = self.cursor_x.saturating_sub(steps);
                        self.ansi_state = AnsiState::Normal;
                    }

                    _ => self.ansi_state = AnsiState::Normal, // конец или неизвестная команда
                }
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    fn toggle_cursor(&mut self) {
        let base_x = self.cursor_x * CHAR_W;
        let base_y = self.cursor_y * CHAR_H;

        for row in 0..CHAR_H {
            for col in 0..CHAR_W {
                let x = base_x + col;
                let y = base_y + row;
                if x >= self.info.width as usize || y >= self.info.height as usize { continue; }
            
                let offset = (y * self.info.stride as usize + x) * 4;
                if offset + 4 <= self.buffer.len() {
                    self.buffer[offset]     = !self.buffer[offset];
                    self.buffer[offset + 1] = !self.buffer[offset + 1];
                    self.buffer[offset + 2] = !self.buffer[offset + 2];
                }
            }
        }
    }

    pub fn show_cursor(&mut self) {
        self.toggle_cursor();
    }

    pub fn hide_cursor(&mut self) {
        self.toggle_cursor();
    }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// глобальный экземпляр
static TERMINAL: spin::Mutex<Option<Terminal>> = spin::Mutex::new(None);

pub fn init(framebuffer: Option<FrameBuffer>) {
    if let Some(fb) = framebuffer {
        let info = fb.info();
        let buffer = fb.into_buffer();
        let mut term = Terminal::new(buffer, info);
        term.show_cursor();
        *TERMINAL.lock() = Some(term);
    }
}

pub fn clear_screen() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(term) = &mut *TERMINAL.lock() {
            term.clear_screen();
            term.show_cursor();
        }
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(term) = &mut *TERMINAL.lock() {
            term.hide_cursor();
            let _ = term.write_fmt(args);
            term.show_cursor();
        }
    });
}

pub fn write_pixel_global(x: usize, y: usize, color_rgb: u32) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(term) = &mut *TERMINAL.lock() {
            term.write_pixel(x, y, color_rgb);
        }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::terminal::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}