use std::{mem::size_of, ptr, slice, thread, time::Duration};

use windows_sys::Win32::Foundation::GlobalFree;
use windows_sys::Win32::{
    Foundation::{HGLOBAL, HWND},
    Graphics::Gdi::{BI_RGB, BITMAPINFOHEADER},
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW,
            SetClipboardData,
        },
        Memory::{GMEM_MOVEABLE, GMEM_ZEROINIT, GlobalAlloc, GlobalLock, GlobalUnlock},
    },
    UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, HWND_MESSAGE},
};

use super::{os_error, screen::Frame, wide};
use crate::geometry::Rect;

const CF_DIB: u32 = 8;

struct Owner(HWND);

impl Owner {
    fn new() -> Result<Self, String> {
        let window = unsafe {
            CreateWindowExW(
                0,
                wide("STATIC").as_ptr(),
                ptr::null(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        };
        if window.is_null() {
            return Err(os_error("Creating the clipboard owner window"));
        }
        Ok(Self(window))
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.0) };
    }
}

struct Clipboard;

impl Clipboard {
    fn open(owner: HWND) -> Result<Self, String> {
        for attempt in 0..6 {
            if unsafe { OpenClipboard(owner) } != 0 {
                return Ok(Self);
            }
            if attempt < 5 {
                thread::sleep(Duration::from_millis(10));
            }
        }
        Err("The Windows clipboard is busy. Try copying again.".into())
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe { CloseClipboard() };
    }
}

struct ClipboardMemory(HGLOBAL);

impl ClipboardMemory {
    fn new(bytes: usize) -> Result<Self, String> {
        let handle = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes) };
        if handle.is_null() {
            return Err(os_error("Allocating clipboard memory"));
        }
        Ok(Self(handle))
    }

    fn publish(mut self, format: u32) -> Result<(), String> {
        if unsafe { SetClipboardData(format, self.0) }.is_null() {
            return Err(os_error("Publishing clipboard data"));
        }
        self.0 = ptr::null_mut();
        Ok(())
    }
}

impl Drop for ClipboardMemory {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { GlobalFree(self.0) };
        }
    }
}

fn dib_header(rect: Rect) -> BITMAPINFOHEADER {
    BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: rect.width(),
        biHeight: rect.height(),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        ..Default::default()
    }
}

fn copy_rows(source: &[u8], source_width: usize, rect: Rect, target: &mut [u8]) {
    let row_bytes = rect.width() as usize * 4;
    let column = rect.left as usize * 4;
    let source_stride = source_width * 4;
    // CF_DIB is bottom-up. A 32-bit row already satisfies DWORD alignment.
    for (y, row) in (rect.top..rect.bottom)
        .rev()
        .zip(target.chunks_exact_mut(row_bytes))
    {
        let offset = y as usize * source_stride + column;
        row.copy_from_slice(&source[offset..offset + row_bytes]);
        for pixel in row.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
    }
}

pub fn copy(frame: &Frame, requested: Rect) -> Result<(), String> {
    let rect = requested.intersection(frame.image_rect());
    if !rect.has_area() {
        return Err("Select a region before copying.".into());
    }
    publish(rect, |image| {
        copy_rows(
            frame.dib.pixels(),
            frame.bounds.width() as usize,
            rect,
            image,
        );
    })
}

pub fn copy_rgba(width: i32, height: i32, pixels: &[u8]) -> Result<(), String> {
    let rect = Rect {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    };
    if !rect.has_area() || width as u64 * height as u64 * 4 != pixels.len() as u64 {
        return Err("The edited image has invalid dimensions.".into());
    }
    publish(rect, |image| encode_rgba(pixels, width as usize * 4, image))
}

fn encode_rgba(pixels: &[u8], stride: usize, image: &mut [u8]) {
    for (source, target) in pixels
        .chunks_exact(stride)
        .rev()
        .zip(image.chunks_exact_mut(stride))
    {
        for (rgba, bgra) in source.chunks_exact(4).zip(target.chunks_exact_mut(4)) {
            bgra.copy_from_slice(&[rgba[2], rgba[1], rgba[0], 255]);
        }
    }
}

fn publish(rect: Rect, write_pixels: impl FnOnce(&mut [u8])) -> Result<(), String> {
    let header_len = size_of::<BITMAPINFOHEADER>();
    let bytes = rect.width() as usize * rect.height() as usize * 4;
    let memory = ClipboardMemory::new(header_len + bytes)?;
    let data = unsafe { GlobalLock(memory.0) }.cast::<u8>();
    if data.is_null() {
        return Err(os_error("Locking clipboard memory"));
    }
    unsafe {
        data.cast::<BITMAPINFOHEADER>().write(dib_header(rect));
        let image = slice::from_raw_parts_mut(data.add(header_len), bytes);
        write_pixels(image);
        GlobalUnlock(memory.0);
    }
    let privacy_format = unsafe {
        RegisterClipboardFormatW(wide("ExcludeClipboardContentFromMonitorProcessing").as_ptr())
    };
    if privacy_format == 0 {
        return Err(os_error("Registering the clipboard privacy format"));
    }
    let privacy = ClipboardMemory::new(size_of::<u32>())?;
    let owner = Owner::new()?;
    let _clipboard = Clipboard::open(owner.0)?;
    if unsafe { EmptyClipboard() } == 0 {
        return Err(os_error("Clearing the clipboard"));
    }
    privacy.publish(privacy_format)?;
    memory.publish(CF_DIB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_copies_only_requested_pixels_in_bottom_up_order() {
        let source = [
            1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 7, 0, 0, 0, 8,
            0, 0, 0, 9, 0, 0, 0,
        ];
        let rect = Rect {
            left: 1,
            top: 0,
            right: 3,
            bottom: 2,
        };
        let mut target = [0; 16];
        copy_rows(&source, 3, rect, &mut target);
        assert_eq!(
            target,
            [5, 0, 0, 255, 6, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255]
        );
        let header = dib_header(rect);
        assert_eq!(
            (header.biWidth, header.biHeight, header.biBitCount),
            (2, 2, 32)
        );
    }

    #[test]
    fn editor_pixels_convert_to_bottom_up_opaque_native_format() {
        let mut dib = [0; 8];
        encode_rgba(&[10, 20, 30, 255, 90, 80, 70, 255], 4, &mut dib);
        assert_eq!(dib, [70, 80, 90, 255, 30, 20, 10, 255]);
    }
}
