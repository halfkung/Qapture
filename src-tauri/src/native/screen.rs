use std::{ffi::c_void, mem::size_of, ptr, slice};

use serde::Serialize;
use windows_sys::Win32::{
    Foundation::{LPARAM, POINT, RECT},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleDC,
        CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, EnumDisplayMonitors, GdiFlush,
        GetDC, GetMonitorInfoW, HBITMAP, HDC, HGDIOBJ, HMONITOR, MONITORINFO, ReleaseDC, SRCCOPY,
        SelectObject,
    },
    UI::{
        HiDpi::{
            DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor,
            MDT_EFFECTIVE_DPI, SetThreadDpiAwarenessContext,
        },
        WindowsAndMessaging::{GetPhysicalCursorPos, MONITORINFOF_PRIMARY},
    },
};

use super::os_error;
use crate::geometry::{Point, Rect};

const MAX_DESKTOP_PIXELS: i64 = 64 * 1024 * 1024;

pub struct PhysicalDpi(DPI_AWARENESS_CONTEXT);

impl PhysicalDpi {
    pub fn enter() -> Result<Self, String> {
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.is_null() {
            return Err(os_error("Setting physical DPI awareness"));
        }
        Ok(Self(previous))
    }
}

impl Drop for PhysicalDpi {
    fn drop(&mut self) {
        unsafe { SetThreadDpiAwarenessContext(self.0) };
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Display {
    pub bounds: Rect,
    pub dpi: u32,
    pub primary: bool,
}

impl From<RECT> for Rect {
    fn from(rect: RECT) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

impl From<Rect> for RECT {
    fn from(rect: Rect) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

unsafe extern "system" fn collect_monitor(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    data: LPARAM,
) -> i32 {
    let displays = unsafe { &mut *(data as *mut Vec<Display>) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return 0;
    }
    let bounds = Rect::from(info.rcMonitor);
    if bounds.has_area() {
        let (mut dpi_x, mut dpi_y) = (96, 96);
        let result =
            unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) };
        displays.push(Display {
            bounds,
            dpi: if result >= 0 { dpi_x } else { 96 },
            primary: info.dwFlags & MONITORINFOF_PRIMARY != 0,
        });
    }
    1
}

pub fn displays() -> Result<Vec<Display>, String> {
    let _dpi = PhysicalDpi::enter()?;
    let mut displays = Vec::<Display>::new();
    let success = unsafe {
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null(),
            Some(collect_monitor),
            &mut displays as *mut Vec<Display> as LPARAM,
        )
    };
    if success == 0 || displays.is_empty() {
        return Err("Windows could not enumerate a usable display topology.".into());
    }
    Ok(displays)
}

pub fn cursor() -> Option<Point> {
    let mut point = POINT::default();
    (unsafe { GetPhysicalCursorPos(&mut point) } != 0).then_some(Point {
        x: point.x,
        y: point.y,
    })
}

struct DesktopDc(HDC);

impl Drop for DesktopDc {
    fn drop(&mut self) {
        unsafe { ReleaseDC(ptr::null_mut(), self.0) };
    }
}

pub struct Dib {
    dc: HDC,
    image: HBITMAP,
    previous: HGDIOBJ,
    data: *mut u8,
    bytes: usize,
}

impl Dib {
    pub fn new(width: i32, height: i32) -> Result<Self, String> {
        let area = i64::from(width) * i64::from(height);
        if width <= 0 || height <= 0 || area > MAX_DESKTOP_PIXELS {
            return Err("The virtual desktop exceeds Qapture’s capture size limit.".into());
        }
        let mut dib = Self {
            dc: unsafe { CreateCompatibleDC(ptr::null_mut()) },
            image: ptr::null_mut(),
            previous: ptr::null_mut(),
            data: ptr::null_mut(),
            bytes: area as usize * 4,
        };
        if dib.dc.is_null() {
            return Err(os_error("Creating a GDI memory context"));
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels: *mut c_void = ptr::null_mut();
        dib.image = unsafe {
            CreateDIBSection(
                dib.dc,
                &info,
                DIB_RGB_COLORS,
                &mut pixels,
                ptr::null_mut(),
                0,
            )
        };
        if dib.image.is_null() || pixels.is_null() {
            return Err(os_error("Allocating the capture bitmap"));
        }
        dib.data = pixels.cast();
        dib.previous = unsafe { SelectObject(dib.dc, dib.image) };
        if dib.previous.is_null() {
            return Err(os_error("Selecting the capture bitmap"));
        }
        Ok(dib)
    }

    pub fn dc(&self) -> HDC {
        self.dc
    }

    pub fn pixels(&self) -> &[u8] {
        // GDI and this view are confined to the capture worker. Flush pending writes first.
        unsafe {
            GdiFlush();
            slice::from_raw_parts(self.data, self.bytes)
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            GdiFlush();
            ptr::write_bytes(self.data, 0, self.bytes);
        }
    }

    fn pixels_mut(&mut self) -> &mut [u8] {
        // The DIB has been initialized and its GDI operations run on this worker.
        unsafe {
            GdiFlush();
            slice::from_raw_parts_mut(self.data, self.bytes)
        }
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        unsafe {
            if !self.previous.is_null() {
                SelectObject(self.dc, self.previous);
            }
            if !self.dc.is_null() {
                DeleteDC(self.dc);
            }
            if !self.image.is_null() {
                DeleteObject(self.image);
            }
        }
    }
}

pub struct Frame {
    pub bounds: Rect,
    pub displays: Vec<Display>,
    pub dib: Dib,
}

impl Frame {
    pub fn image_rect(&self) -> Rect {
        self.bounds.local_rect(self.bounds)
    }

    pub fn dimmed(&self) -> Result<Dib, String> {
        let mut shadow = Dib::new(self.bounds.width(), self.bounds.height())?;
        shadow.clear();
        for (source, target) in self
            .dib
            .pixels()
            .chunks_exact(4)
            .zip(shadow.pixels_mut().chunks_exact_mut(4))
        {
            let pixel = u32::from_le_bytes(source.try_into().unwrap());
            target.copy_from_slice(&dim_pixel(pixel).to_le_bytes());
        }
        Ok(shadow)
    }
}

fn dim_pixel(pixel: u32) -> u32 {
    // Scale each BGR channel to roughly 37.5%, leaving no shared alpha byte.
    ((pixel >> 2) & 0x003f3f3f) + ((pixel >> 3) & 0x001f1f1f)
}

pub fn capture() -> Result<Frame, String> {
    let displays = displays()?;
    let bounds = displays
        .iter()
        .map(|display| display.bounds)
        .reduce(Rect::union)
        .ok_or("Windows reported no displays.")?;
    let screen = DesktopDc(unsafe { GetDC(ptr::null_mut()) });
    if screen.0.is_null() {
        return Err(os_error("Opening the desktop device context"));
    }
    let mut dib = Dib::new(bounds.width(), bounds.height())?;
    dib.clear();

    // Copy monitor rectangles individually so gaps retain their black fill.
    for display in &displays {
        let target = bounds.local_rect(display.bounds);
        let ok = unsafe {
            BitBlt(
                dib.dc(),
                target.left,
                target.top,
                target.width(),
                target.height(),
                screen.0,
                display.bounds.left,
                display.bounds.top,
                SRCCOPY | CAPTUREBLT,
            )
        };
        if ok == 0 {
            return Err(os_error("Capturing the Windows desktop"));
        }
    }
    Ok(Frame {
        bounds,
        displays,
        dib,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shade_darkens_each_color_channel_independently() {
        assert_eq!(dim_pixel(0xaa_ff_80_08), 0x00_5e_30_03);
        assert_eq!(dim_pixel(0xff_00_00_00), 0);
    }
}
