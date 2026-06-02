use alloc::vec::Vec;
use alloc::vec;
use bootloader_api::info::{PixelFormat, FrameBufferInfo};

pub struct GopDisplay {
    front_buffer: &'static mut [u8],
    back_buffer: Vec<u8>,
    pub info: FrameBufferInfo,
    raw_bg_bytes: [u8; 4],
    bg_color: u32,
}

impl GopDisplay {
    pub fn new(front_buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let back_buffer = vec![0; front_buffer.len()];
        let mut display = Self {
            front_buffer,
            back_buffer,
            info,
            raw_bg_bytes: [0; 4],
            bg_color: 0,
        };
        display.update_bg_cache(0);
        display.clear_screen();
        display.swap_buffers();
        display
    }

    pub fn update_bg_cache(&mut self, bg_color: u32) {
        self.bg_color = bg_color;
        self.raw_bg_bytes = match self.info.pixel_format {
            PixelFormat::Rgb => bg_color.to_be_bytes(),
            PixelFormat::Bgr => {
                let r = (bg_color >> 16) & 0xFF;
                let g = (bg_color >> 8) & 0xFF;
                let b = bg_color & 0xFF;
                (b << 16 | g << 8 | r).to_le_bytes()
            },
            _ => bg_color.to_le_bytes(),
        };
    }

    #[inline(always)]
    pub fn write_pixel(&mut self, x: usize, y: usize, color_rgb: u32) {
        if x >= self.info.width as usize || y >= self.info.height as usize { return; }
        let offset = (y * self.info.stride as usize + x) * 4;
        
        let bytes = if color_rgb == self.bg_color {
            self.raw_bg_bytes
        } else {
            match self.info.pixel_format {
                PixelFormat::Rgb => color_rgb.to_be_bytes(),
                PixelFormat::Bgr => {
                    let r = (color_rgb >> 16) & 0xFF;
                    let g = (color_rgb >> 8) & 0xFF;
                    let b = color_rgb & 0xFF;
                    (b << 16 | g << 8 | r).to_le_bytes()
                },
                _ => color_rgb.to_le_bytes(),
            }
        };

        if offset + 4 <= self.back_buffer.len() {
            self.back_buffer[offset..offset + 4].copy_from_slice(&bytes);
        }
    }

    pub fn swap_buffers(&mut self) {
        let src = &self.back_buffer[..];
        let dst = &mut self.front_buffer[..];
        
        let (prefix, src_u32, suffix) = unsafe { src.align_to::<u32>() };
        let (d_prefix, dst_u32, d_suffix) = unsafe { dst.align_to_mut::<u32>() };
        
        if prefix.is_empty() && d_prefix.is_empty() && src_u32.len() == dst_u32.len() {
            dst_u32.copy_from_slice(src_u32);
        } else {
            dst.copy_from_slice(src);
        }
    }

    pub fn clear_screen(&mut self) {
        let c_u32 = u32::from_ne_bytes(self.raw_bg_bytes);
        let buf = &mut self.back_buffer[..];
        let (_, buf_u32, _) = unsafe { buf.align_to_mut::<u32>() };
        buf_u32.fill(c_u32);
    }

    pub fn scroll(&mut self, char_h: usize) {
        let row_bytes = self.info.stride as usize * 4;
        let total_height = self.info.height as usize;
        let text_row_offset = char_h * row_bytes;
        let buffer_end = total_height * row_bytes;

        if buffer_end > text_row_offset {
            self.back_buffer.copy_within(text_row_offset..buffer_end, 0);
        }

        let c_u32 = u32::from_ne_bytes(self.raw_bg_bytes);
        let clear_start = buffer_end - text_row_offset;
        let clear_slice = &mut self.back_buffer[clear_start..buffer_end];
        let (_, clear_u32, _) = unsafe { clear_slice.align_to_mut::<u32>() };
        clear_u32.fill(c_u32);
    }

    pub fn toggle_cursor_pixels(&mut self, base_x: usize, base_y: usize, width: usize, height: usize) {
        for row in 0..height {
            for col in 0..width {
                let x = base_x + col;
                let y = base_y + row;
                if x >= self.info.width as usize || y >= self.info.height as usize { continue; }
            
                let offset = (y * self.info.stride as usize + x) * 4;
                if offset + 4 <= self.back_buffer.len() {
                    self.back_buffer[offset]     = !self.back_buffer[offset];
                    self.back_buffer[offset + 1] = !self.back_buffer[offset + 1];
                    self.back_buffer[offset + 2] = !self.back_buffer[offset + 2];
                }
            }
        }
    }
}