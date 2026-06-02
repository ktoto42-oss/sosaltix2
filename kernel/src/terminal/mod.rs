pub mod framebuffer;
pub mod emulator;

use core::fmt;
use core::fmt::Write;
use bootloader_api::info::FrameBuffer;
pub use emulator::Terminal;
use framebuffer::GopDisplay;

static TERMINAL: spin::Mutex<Option<Terminal>> = spin::Mutex::new(None);

pub fn init(framebuffer: Option<FrameBuffer>) {
    if let Some(fb) = framebuffer {
        let info = fb.info();
        let buffer = fb.into_buffer();
        let display = GopDisplay::new(buffer, info);
        let mut term = Terminal::new(display);
        term.show_cursor();
        term.display.swap_buffers();
        *TERMINAL.lock() = Some(term);
    }
}

pub fn clear_screen() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(term) = &mut *TERMINAL.lock() {
            term.clear_screen();
            term.show_cursor();
            term.display.swap_buffers();
        }
    });
}

pub fn redraw() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(term) = &mut *TERMINAL.lock() {
            term.display.swap_buffers();
        }
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
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
            term.display.write_pixel(x, y, color_rgb);
            term.display.swap_buffers();
            redraw();
        }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::terminal::_print(format_args!($($arg)*));
        $crate::terminal::redraw();
    }
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
        $crate::terminal::redraw();
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
        $crate::terminal::redraw();
    };
}