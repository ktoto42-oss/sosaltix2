use core::fmt;
use crate::font::FONT;
use super::framebuffer::GopDisplay;

pub const CHAR_W: usize = 8;
pub const CHAR_H: usize = 16;

const COLORS: [u32; 16] = [
    0x000000, 0xAA0000, 0x00AA00, 0xAA5500, 0x0000AA, 0xAA00AA, 0x00AAAA, 0xAAAAAA,
    0x555555, 0xFF5555, 0x55FF55, 0xFFFF55, 0x5555FF, 0xFF55FF, 0x55FFFF, 0xFFFFFF,
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnsiState {
    Normal,
    Escaped,
    ParsingCsi,
}

pub struct Terminal {
    pub display: GopDisplay,
    cols: usize,
    rows: usize,
    cursor_x: usize,
    cursor_y: usize,
    fg_color: u32,
    bg_color: u32,
    ansi_state: AnsiState,
    ansi_param: usize,
}

impl Terminal {
    pub fn new(display: GopDisplay) -> Self {
        let cols = display.info.width as usize / CHAR_W;
        let rows = display.info.height as usize / CHAR_H;
        
        let mut term = Self {
            display,
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            fg_color: COLORS[7],
            bg_color: COLORS[0],
            ansi_state: AnsiState::Normal,
            ansi_param: 0,
        };
        term.display.update_bg_cache(term.bg_color);
        term.clear_screen();
        term
    }

    pub fn clear_screen(&mut self) {
        self.display.clear_screen();
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
                self.display.write_pixel(base_x + col, base_y + row, color);
            }
        }
    }

    fn scroll(&mut self) {
        self.display.scroll(CHAR_H);
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }

    fn new_line(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.scroll();
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match self.ansi_state {
            AnsiState::Normal => {
                match byte {
                    0x1B => self.ansi_state = AnsiState::Escaped,
                    b'\n' => self.new_line(),
                    b'\r' => self.cursor_x = 0,
                    b'\x08' => {
                        if self.cursor_x > 0 {
                            self.cursor_x -= 1;
                            self.draw_char(self.cursor_x, self.cursor_y, b' ');
                        }
                    }
                    0x20..=0x7E => {
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
                    _ => self.ansi_state = AnsiState::Normal,
                }
            }
            AnsiState::ParsingCsi => {
                match byte {
                    b'0'..=b'9' => {
                        self.ansi_param = self.ansi_param * 10 + (byte - b'0') as usize;
                    }
                    b'm' => {
                        match self.ansi_param {
                            0 => {
                                self.fg_color = COLORS[7];
                                self.bg_color = COLORS[0];
                            }
                            30..=37 => self.fg_color = COLORS[self.ansi_param - 30],
                            90..=97 => self.fg_color = COLORS[self.ansi_param - 90 + 8],
                            40..=47 => self.bg_color = COLORS[self.ansi_param - 40],
                            100..=107 => self.bg_color = COLORS[self.ansi_param - 100 + 8],
                            _ => {}
                        }
                        self.display.update_bg_cache(self.bg_color);
                        self.ansi_state = AnsiState::Normal;
                    }
                    b'C' => {
                        let steps = if self.ansi_param == 0 { 1 } else { self.ansi_param };
                        self.cursor_x = (self.cursor_x + steps).min(self.cols - 1);
                        self.ansi_state = AnsiState::Normal;
                    }
                    b'D' => {
                        let steps = if self.ansi_param == 0 { 1 } else { self.ansi_param };
                        self.cursor_x = self.cursor_x.saturating_sub(steps);
                        self.ansi_state = AnsiState::Normal;
                    }
                    _ => self.ansi_state = AnsiState::Normal,
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
        self.display.toggle_cursor_pixels(
            self.cursor_x * CHAR_W,
            self.cursor_y * CHAR_H,
            CHAR_W,
            CHAR_H
        );
    }

    pub fn show_cursor(&mut self) { self.toggle_cursor(); }
    pub fn hide_cursor(&mut self) { self.toggle_cursor(); }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}