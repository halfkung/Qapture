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
            BeginPaint, BitBlt, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DT_CENTER,
            DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DeleteObject, DrawTextW, EndPaint, FillRect,
            FrameRect, HDC, HFONT, InvalidateRect, PAINTSTRUCT, SRCCOPY, SelectObject, SetBkMode,
            SetTextColor, StretchBlt, TRANSPARENT, UpdateWindow,
        },
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::{ReleaseCapture, SetCapture, SetFocus, VK_ESCAPE, VK_RETURN},
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
const BORDER: u32 = 0xffe3ce;

struct Chrome {
    panel: Rect,
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
        Ok(Self { panel, font })
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
    operation: Operation,
}

#[derive(Clone, Copy)]
enum Operation {
    Draw,
    Move(Rect),
    Resize(Rect),
}

impl Drag {
    fn rect(self, bounds: Rect) -> Rect {
        match self.operation {
            Operation::Draw => Rect::between(self.start, self.current),
            Operation::Move(rect) => {
                let dx =
                    (self.current.x - self.start.x).clamp(-rect.left, bounds.right - rect.right);
                let dy =
                    (self.current.y - self.start.y).clamp(-rect.top, bounds.bottom - rect.bottom);
                Rect {
                    left: rect.left + dx,
                    right: rect.right + dx,
                    top: rect.top + dy,
                    bottom: rect.bottom + dy,
                }
            }
            Operation::Resize(rect) => Rect {
                right: (rect.right + self.current.x - self.start.x)
                    .clamp(rect.left + 1, bounds.right),
                bottom: (rect.bottom + self.current.y - self.start.y)
                    .clamp(rect.top + 1, bounds.bottom),
                ..rect
            },
        }
    }
}

struct Overlay<'a> {
    frame: &'a Frame,
    cancelled: &'a AtomicBool,
    shade: Dib,
    chrome: Chrome,
    drag: Cell<Option<Drag>>,
    selected: Cell<Option<Rect>>,
    adjust: bool,
    windows: Option<Vec<Rect>>,
    hover: Cell<Option<Rect>>,
    magnifier: Cell<bool>,
    pointer: Cell<Point>,
    result: Cell<Option<Rect>>,
}

impl Overlay<'_> {
    fn visible_selection(&self) -> Option<Rect> {
        if self.windows.is_some() {
            return self.hover.get();
        }
        self.drag
            .get()
            .map(|drag| drag.rect(self.frame.image_rect()))
            .or_else(|| self.selected.get())
            .filter(|rect| rect.has_area())
    }

    fn confirm(&self, window: HWND, rect: Rect) {
        self.result.set(Some(rect));
        unsafe { DestroyWindow(window) };
    }

    fn track(&self, window: HWND, point: Point) {
        let before = self.visible_selection();
        let magnifier = self.magnifier_rect();
        self.pointer.set(point);
        if let Some(windows) = &self.windows {
            self.hover.set(
                windows
                    .iter()
                    .copied()
                    .find(|bounds| bounds.contains(point)),
            );
        }
        if let Some(drag) = self.drag.get() {
            self.drag.set(Some(Drag {
                current: point,
                ..drag
            }));
        }
        self.invalidate(window, before);
        if self.magnifier.get() {
            unsafe {
                InvalidateRect(window, &magnifier.into(), 0);
                InvalidateRect(window, &self.magnifier_rect().into(), 0);
            }
        }
    }

    fn magnifier_rect(&self) -> Rect {
        let point = self.pointer.get();
        let display = self
            .frame
            .displays
            .iter()
            .map(|display| self.frame.bounds.local_rect(display.bounds))
            .find(|rect| rect.contains(point))
            .unwrap_or(self.frame.image_rect());
        let size = 96.min(display.width()).min(display.height());
        let x = if point.x + 24 + size <= display.right {
            point.x + 24
        } else {
            point.x - 24 - size
        };
        let left = x.clamp(display.left, display.right - size);
        let top = (point.y + 24).clamp(display.top, display.bottom - size);
        Rect {
            left,
            top,
            right: left + size,
            bottom: top + size,
        }
    }

    fn paint_magnifier(&self, dc: HDC) {
        let glass = self.magnifier_rect();
        let point = self.pointer.get();
        let bounds = self.frame.image_rect();
        let width = 15.min(bounds.width());
        let height = 15.min(bounds.height());
        fill(dc, glass, PANEL_BG, false);
        unsafe {
            StretchBlt(
                dc,
                glass.left,
                glass.top,
                glass.width(),
                glass.height(),
                self.frame.dib.dc(),
                (point.x - 7).clamp(0, bounds.right - width),
                (point.y - 7).clamp(0, bounds.bottom - height),
                width,
                height,
                SRCCOPY,
            );
        }
        fill(dc, glass, BORDER, true);
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
                blit(dc, self.shade.dc(), dirty);
                let selection = self.visible_selection();
                if let Some(rect) = selection {
                    let exposed = rect.intersection(dirty);
                    if exposed.has_area() {
                        blit(dc, self.frame.dib.dc(), exposed);
                    }
                    fill(dc, rect, BORDER, true);
                    if self.adjust && self.windows.is_none() {
                        fill(
                            dc,
                            Rect {
                                left: rect.right - 7,
                                top: rect.bottom - 7,
                                right: rect.right,
                                bottom: rect.bottom,
                            },
                            BORDER,
                            false,
                        );
                    }
                }
                if self.chrome.panel.intersection(dirty).has_area() {
                    self.paint_panel(dc, selection);
                }
                if self.magnifier.get() && self.magnifier_rect().intersection(dirty).has_area() {
                    self.paint_magnifier(dc);
                }
            }
        }
        unsafe { EndPaint(window, &paint) };
    }

    fn paint_panel(&self, dc: HDC, selection: Option<Rect>) {
        let panel = self.chrome.panel;
        fill(dc, panel, PANEL_BG, false);
        let message = match (selection, self.windows.is_some(), self.adjust) {
            (Some(rect), true, _) => {
                format!("{} × {} px  ·  Click to edit", rect.width(), rect.height())
            }
            (Some(rect), _, true) => format!(
                "{} × {} px  ·  Move / resize  ·  Enter to edit",
                rect.width(),
                rect.height()
            ),
            (Some(rect), _, _) => format!(
                "{} × {} px  ·  Release to edit",
                rect.width(),
                rect.height()
            ),
            (None, true, _) => "Point at a window to select it  ·  Esc to cancel".into(),
            _ => "Drag to select  ·  M to magnify  ·  Esc to cancel".into(),
        };
        let old = unsafe { SelectObject(dc, self.chrome.font) };
        unsafe {
            SetBkMode(dc, TRANSPARENT as i32);
            SetTextColor(dc, PANEL_FG);
        }
        draw_text(dc, panel, &message);
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
            if let Some(windows) = &state.windows {
                if let Some(rect) = windows
                    .iter()
                    .copied()
                    .find(|bounds| bounds.contains(point))
                {
                    state.confirm(window, rect);
                }
                return 0;
            }
            let before = state.visible_selection();
            if let Some(rect) = state.selected.get()
                && state.chrome.panel.contains(point)
            {
                state.confirm(window, rect);
                return 0;
            }
            let operation = state.selected.get().map_or(Operation::Draw, |rect| {
                if (point.x - rect.right).abs() <= 12 && (point.y - rect.bottom).abs() <= 12 {
                    Operation::Resize(rect)
                } else if rect.contains(point) {
                    Operation::Move(rect)
                } else {
                    Operation::Draw
                }
            });
            state.drag.set(Some(Drag {
                start: point,
                current: point,
                operation,
            }));
            unsafe { SetCapture(window) };
            state.invalidate(window, before);
        }
        WM_MOUSEMOVE => {
            let point = state.point(lparam);
            if point != state.pointer.get()
                && (state.drag.get().is_some() || state.windows.is_some() || state.magnifier.get())
            {
                state.track(window, point);
            } else {
                state.pointer.set(point);
            }
        }
        WM_LBUTTONUP => {
            let before = state.visible_selection();
            if let Some(drag) = state.drag.take() {
                let selection = Drag {
                    current: state.point(lparam),
                    ..drag
                }
                .rect(state.frame.image_rect());
                unsafe { ReleaseCapture() };
                if selection.has_area() && !state.adjust {
                    state.confirm(window, selection);
                } else {
                    if selection.has_area() {
                        state.selected.set(Some(selection));
                    }
                    state.invalidate(window, before);
                }
            }
        }
        WM_CAPTURECHANGED => {
            let before = state.visible_selection();
            if state.drag.take().is_some() {
                state.invalidate(window, before);
            }
        }
        WM_KEYDOWN if wparam == usize::from(VK_ESCAPE) => {
            unsafe { DestroyWindow(window) };
        }
        WM_KEYDOWN if wparam == usize::from(VK_RETURN) => {
            if let Some(rect) = state.selected.get().or_else(|| state.hover.get()) {
                state.confirm(window, rect);
            }
        }
        WM_KEYDOWN if wparam == b'M' as usize && (lparam & (1 << 30)) == 0 => {
            state.magnifier.set(!state.magnifier.get());
            unsafe { InvalidateRect(window, &state.magnifier_rect().into(), 0) };
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
    adjust: bool,
    windows: Option<Vec<Rect>>,
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
    let pointer = bounds.local(screen::cursor().unwrap_or(Point {
        x: bounds.left,
        y: bounds.top,
    }));
    let hover = windows
        .as_ref()
        .and_then(|rects| rects.iter().copied().find(|rect| rect.contains(pointer)));
    let state = Overlay {
        frame,
        cancelled,
        shade: frame.dimmed()?,
        chrome: Chrome::new(frame)?,
        drag: Cell::new(None),
        selected: Cell::new(None),
        adjust,
        windows,
        hover: Cell::new(hover),
        magnifier: Cell::new(false),
        pointer: Cell::new(pointer),
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
