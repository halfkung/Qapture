use std::{
    cell::Cell,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::{
        Dwm::DwmFlush,
        Gdi::{
            AC_SRC_OVER, AlphaBlend, BLENDFUNCTION, BeginPaint, BitBlt, CreateFontW,
            CreateSolidBrush, DEFAULT_CHARSET, DT_CENTER, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER,
            DeleteObject, DrawTextW, EndPaint, FillRect, FrameRect, HDC, HFONT, InvalidateRect,
            PAINTSTRUCT, SRCCOPY, SelectObject, SetBkMode, SetTextColor, TRANSPARENT, UpdateWindow,
        },
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::{
            GetKeyState, ReleaseCapture, SetCapture, SetFocus, VK_CONTROL, VK_ESCAPE, VK_RETURN,
        },
        WindowsAndMessaging::*,
    },
};

use super::{
    os_error,
    screen::{self, Dib, Frame},
    wide,
};
use crate::geometry::{Point, Rect};

static CLASS_REGISTERED: OnceLock<Result<(), String>> = OnceLock::new();
const CLASS_NAME: &str = "QaptureSelectionOverlay";
const CANCEL_TIMER: usize = 1;
const PANEL_BG: u32 = 0x24201c;
const PANEL_FG: u32 = 0xfff8f4;
const ACCENT: u32 = 0xe85462;
const BORDER: u32 = 0xffc5b5;

struct Chrome {
    panel: Rect,
    button: Rect,
    font: HFONT,
}

impl Chrome {
    fn new(frame: &Frame) -> Result<Self, String> {
        let cursor = screen::cursor();
        let display = frame
            .displays
            .iter()
            .find(|item| cursor.is_some_and(|point| item.bounds.contains(point)))
            .or_else(|| frame.displays.iter().find(|item| item.primary))
            .unwrap_or(&frame.displays[0]);
        let bounds = frame.bounds.local_rect(display.bounds);
        // Scale only the controls. Selection coordinates and painting remain physical.
        let px = |logical: i32| (logical * display.dpi as i32 + 48) / 96;
        let panel_width = px(454).min(bounds.width());
        let left = bounds.left + (bounds.width() - panel_width) / 2;
        let top = bounds.top + px(22);
        let panel = Rect {
            left,
            top,
            right: left + panel_width,
            bottom: top + px(52),
        };
        let button = Rect {
            left: panel.right - px(112).min(panel_width / 3),
            right: panel.right - px(8),
            top: panel.top + px(8),
            bottom: panel.bottom - px(8),
        };
        let font = unsafe {
            CreateFontW(
                -px(15),
                0,
                0,
                0,
                500,
                0,
                0,
                0,
                u32::from(DEFAULT_CHARSET),
                0,
                0,
                0,
                0,
                wide("Segoe UI").as_ptr(),
            )
        };
        if font.is_null() {
            return Err(os_error("Creating the selection overlay font"));
        }
        Ok(Self {
            panel,
            button,
            font,
        })
    }
}

impl Drop for Chrome {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.font) };
    }
}

#[derive(Clone, Copy)]
struct Drag {
    start: Point,
    current: Point,
}

impl Drag {
    fn rect(self) -> Rect {
        Rect::between(self.start, self.current)
    }
}

struct Overlay<'a> {
    frame: &'a Frame,
    cancelled: &'a AtomicBool,
    shade: Dib,
    chrome: Chrome,
    drag: Cell<Option<Drag>>,
    selection: Cell<Option<Rect>>,
    result: Cell<Option<Rect>>,
}

impl Overlay<'_> {
    fn visible_selection(&self) -> Option<Rect> {
        self.drag
            .get()
            .map(Drag::rect)
            .or(self.selection.get())
            .filter(|rect| rect.has_area())
    }

    fn invalidate(&self, window: HWND, before: Option<Rect>) {
        for rect in before.into_iter().chain(self.visible_selection()) {
            let dirty = RECT {
                left: rect.left - 2,
                top: rect.top - 2,
                right: rect.right + 2,
                bottom: rect.bottom + 2,
            };
            unsafe { InvalidateRect(window, &dirty, 0) };
        }
        unsafe { InvalidateRect(window, &self.chrome.panel.into(), 0) };
    }

    fn copy_and_close(&self, window: HWND) {
        if let Some(rect) = self.selection.get() {
            self.result.set(Some(rect));
            unsafe { DestroyWindow(window) };
        }
    }

    fn point(&self, value: LPARAM) -> Point {
        let point = Point {
            x: value as i16 as i32,
            y: (value >> 16) as i16 as i32,
        };
        self.frame.image_rect().clamp_edge(point)
    }

    fn paint(&self, window: HWND) {
        let mut paint = PAINTSTRUCT::default();
        let dc = unsafe { BeginPaint(window, &mut paint) };
        if !dc.is_null() {
            let dirty = Rect::from(paint.rcPaint);
            if dirty.has_area() {
                blit(dc, self.frame.dib.dc(), dirty);
                unsafe {
                    AlphaBlend(
                        dc,
                        dirty.left,
                        dirty.top,
                        dirty.width(),
                        dirty.height(),
                        self.shade.dc(),
                        0,
                        0,
                        1,
                        1,
                        BLENDFUNCTION {
                            BlendOp: AC_SRC_OVER as u8,
                            BlendFlags: 0,
                            SourceConstantAlpha: 105,
                            AlphaFormat: 0,
                        },
                    );
                }
                let selection = self.visible_selection();
                if let Some(rect) = selection {
                    let exposed = rect.intersection(dirty);
                    if exposed.has_area() {
                        blit(dc, self.frame.dib.dc(), exposed);
                    }
                    fill(dc, rect, BORDER, true);
                }
                if self.chrome.panel.intersection(dirty).has_area() {
                    self.paint_panel(dc, selection);
                }
            }
        }
        unsafe { EndPaint(window, &paint) };
    }

    fn paint_panel(&self, dc: HDC, selection: Option<Rect>) {
        let panel = self.chrome.panel;
        fill(dc, panel, PANEL_BG, false);
        let committed = self.selection.get().is_some() && self.drag.get().is_none();
        let message = selection.map_or_else(
            || String::from("Drag to select a region   ·   Esc to cancel"),
            |rect| {
                format!(
                    "{} × {} px   ·   Esc to cancel",
                    rect.width(),
                    rect.height()
                )
            },
        );
        let old = unsafe { SelectObject(dc, self.chrome.font) };
        unsafe {
            SetBkMode(dc, TRANSPARENT as i32);
            SetTextColor(dc, PANEL_FG);
        }
        let mut label = panel;
        if committed {
            label.right = self.chrome.button.left;
            fill(dc, self.chrome.button, ACCENT, false);
            draw_text(dc, self.chrome.button, "Copy  ↵");
        }
        draw_text(dc, label, &message);
        unsafe { SelectObject(dc, old) };
    }
}

fn draw_text(dc: HDC, rect: Rect, message: &str) {
    unsafe {
        DrawTextW(
            dc,
            wide(message).as_ptr(),
            -1,
            &mut rect.into(),
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
        );
    }
}

fn fill(dc: HDC, rect: Rect, color: u32, outline: bool) {
    unsafe {
        let brush = CreateSolidBrush(color);
        if !brush.is_null() {
            if outline {
                FrameRect(dc, &rect.into(), brush);
            } else {
                FillRect(dc, &rect.into(), brush);
            }
            DeleteObject(brush);
        }
    }
}

fn blit(to: HDC, from: HDC, rect: Rect) {
    unsafe {
        BitBlt(
            to,
            rect.left,
            rect.top,
            rect.width(),
            rect.height(),
            from,
            rect.left,
            rect.top,
            SRCCOPY,
        );
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as isize) };
    }
    // The worker keeps this shared state alive until the window is destroyed.
    let state =
        unsafe { (GetWindowLongPtrW(window, GWLP_USERDATA) as *const Overlay<'_>).as_ref() };
    let Some(state) = state else {
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    };
    match message {
        WM_PAINT => state.paint(window),
        WM_ERASEBKGND => return 1,
        WM_LBUTTONDOWN => {
            let point = state.point(lparam);
            if state.selection.get().is_some() && state.chrome.button.contains(point) {
                state.copy_and_close(window);
            } else {
                let before = state.visible_selection();
                state.selection.set(None);
                state.drag.set(Some(Drag {
                    start: point,
                    current: point,
                }));
                unsafe { SetCapture(window) };
                state.invalidate(window, before);
            }
        }
        WM_MOUSEMOVE => {
            if let Some(drag) = state.drag.get() {
                let point = state.point(lparam);
                if point != drag.current {
                    let before = state.visible_selection();
                    state.drag.set(Some(Drag {
                        current: point,
                        ..drag
                    }));
                    state.invalidate(window, before);
                }
            }
        }
        WM_LBUTTONUP => {
            let before = state.visible_selection();
            if let Some(drag) = state.drag.take() {
                let selection = Rect::between(drag.start, state.point(lparam));
                state
                    .selection
                    .set(selection.has_area().then_some(selection));
                unsafe { ReleaseCapture() };
                state.invalidate(window, before);
            }
        }
        WM_CAPTURECHANGED => {
            let before = state.visible_selection();
            if state.drag.take().is_some() {
                state.invalidate(window, before);
            }
        }
        WM_KEYDOWN
            if wparam == usize::from(VK_RETURN)
                || (wparam == b'C' as usize
                    && unsafe { GetKeyState(i32::from(VK_CONTROL)) } < 0) =>
        {
            state.copy_and_close(window);
        }
        WM_KEYDOWN if wparam == usize::from(VK_ESCAPE) => {
            unsafe { DestroyWindow(window) };
        }
        WM_TIMER if state.cancelled.load(Ordering::Acquire) => {
            unsafe { DestroyWindow(window) };
        }
        WM_CLOSE | WM_DISPLAYCHANGE | WM_RBUTTONDOWN => {
            unsafe { DestroyWindow(window) };
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
        }
        WM_NCDESTROY => {
            unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, 0) };
            return unsafe { DefWindowProcW(window, message, wparam, lparam) };
        }
        _ => return unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
    0
}

fn register_class(instance: HINSTANCE) -> Result<(), String> {
    CLASS_REGISTERED
        .get_or_init(|| {
            let name = wide(CLASS_NAME);
            let class = WNDCLASSW {
                hInstance: instance,
                hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_CROSS) },
                lpszClassName: name.as_ptr(),
                lpfnWndProc: Some(window_proc),
                ..Default::default()
            };
            if unsafe { RegisterClassW(&class) } == 0 {
                Err(os_error("Registering the selection overlay"))
            } else {
                Ok(())
            }
        })
        .clone()
}

pub fn select(
    frame: &Frame,
    cancelled: &AtomicBool,
    started: Instant,
    presented: impl FnOnce(Duration),
) -> Result<Option<Rect>, String> {
    let bounds = frame.bounds;
    if bounds.width() >= i32::from(i16::MAX) || bounds.height() >= i32::from(i16::MAX) {
        return Err("The desktop is too wide or tall for region selection.".into());
    }
    let instance = unsafe { GetModuleHandleW(ptr::null()) };
    if instance.is_null() {
        return Err(os_error("Obtaining the overlay application handle"));
    }
    register_class(instance)?;
    let mut shade = Dib::new(1, 1)?;
    shade.clear();
    let state = Overlay {
        frame,
        cancelled,
        shade,
        chrome: Chrome::new(frame)?,
        drag: Cell::new(None),
        selection: Cell::new(None),
        result: Cell::new(None),
    };
    let window = unsafe {
        CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            wide(CLASS_NAME).as_ptr(),
            wide("Qapture region selection").as_ptr(),
            WS_POPUP,
            bounds.left,
            bounds.top,
            bounds.width(),
            bounds.height(),
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            (&state as *const Overlay<'_>).cast(),
        )
    };
    if window.is_null() {
        return Err(os_error("Creating the selection overlay"));
    }
    let result = run_overlay(window, cancelled, started, presented);
    if unsafe { IsWindow(window) } != 0 {
        unsafe { DestroyWindow(window) };
    }
    result.map(|()| state.result.get())
}

fn run_overlay(
    window: HWND,
    cancelled: &AtomicBool,
    started: Instant,
    presented: impl FnOnce(Duration),
) -> Result<(), String> {
    if cancelled.load(Ordering::Acquire) {
        return Ok(());
    }
    unsafe {
        if SetTimer(window, CANCEL_TIMER, 60, None) == 0 {
            return Err(os_error("Starting the selection overlay timer"));
        }
        ShowWindow(window, SW_SHOW);
        SetForegroundWindow(window);
        SetFocus(window);
        UpdateWindow(window);
        DwmFlush();
    }
    presented(started.elapsed());
    let mut message = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
        match result {
            -1 => return Err(os_error("Processing selection input")),
            0 => return Ok(()),
            _ => unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            },
        }
    }
}
