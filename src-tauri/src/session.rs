use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use windows_sys::Win32::Graphics::Dwm::DwmFlush;

use crate::{
    dashboard,
    diagnostics::{Diagnostics, Report, milliseconds},
    editor,
    native::{clipboard, overlay, screen},
};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Region,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Idle,
    Capturing,
    Selecting,
    Editing,
    Exporting,
}

const BUSY: &str = "A capture is already in progress.";

pub struct Sessions {
    phase: Mutex<Phase>,
    cancelled: AtomicBool,
}

struct Lease<'a>(&'a Sessions, bool);

impl Sessions {
    pub fn new() -> Self {
        Self {
            phase: Mutex::new(Phase::Idle),
            cancelled: AtomicBool::new(false),
        }
    }

    fn acquire(&self) -> Result<(), String> {
        let mut phase = self.phase.lock().unwrap();
        if *phase != Phase::Idle {
            return Err(BUSY.into());
        }
        self.cancelled.store(false, Ordering::Release);
        *phase = Phase::Capturing;
        Ok(())
    }

    fn release(&self) {
        *self.phase.lock().unwrap() = Phase::Idle;
    }

    pub fn finish_editing(&self) {
        let mut phase = self.phase.lock().unwrap();
        if *phase == Phase::Editing {
            *phase = Phase::Idle;
        }
    }

    pub fn phase(&self) -> Phase {
        *self.phase.lock().unwrap()
    }

    pub fn cancel(&self) {
        let phase = self.phase.lock().unwrap();
        if matches!(*phase, Phase::Capturing | Phase::Selecting) {
            self.cancelled.store(true, Ordering::Release);
        }
    }

    fn advance(&self, next: Phase) -> bool {
        let mut phase = self.phase.lock().unwrap();
        if self.cancelled.load(Ordering::Acquire) {
            return false;
        }
        debug_assert!(matches!(
            (*phase, next),
            (Phase::Capturing, Phase::Selecting | Phase::Exporting)
                | (Phase::Selecting, Phase::Editing)
        ));
        *phase = next;
        true
    }
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        if self.1 {
            self.0.release();
        }
    }
}

pub fn notify(app: &AppHandle) {
    let _ = app.emit_to("main", "qapture-status", ());
}

pub fn start_background(app: &AppHandle, mode: Mode, settle_desktop: bool) {
    if let Err(message) = start(app, mode, false, settle_desktop)
        && message != BUSY
    {
        app.state::<Diagnostics>().record(Report {
            outcome: "error",
            overlay_ms: None,
            clipboard_ms: None,
            message,
        });
        notify(app);
    }
}

pub fn start(
    app: &AppHandle,
    mode: Mode,
    hide_dashboard: bool,
    settle_desktop: bool,
) -> Result<(), String> {
    let started = Instant::now();
    let dashboard = hide_dashboard
        .then(|| app.get_webview_window("main"))
        .flatten();
    let sessions = app.state::<Sessions>();
    sessions.acquire()?;
    if let Some(window) = &dashboard
        && let Err(error) = window.hide()
    {
        sessions.release();
        return Err(format!("Could not hide Qapture before capture ({error})."));
    }
    let worker_dashboard = dashboard.clone();
    let app_worker = app.clone();
    let thread = std::thread::Builder::new()
        .name("qapture-capture".into())
        .spawn(move || {
            let sessions = app_worker.state::<Sessions>();
            let mut lease = Lease(&sessions, true);
            let from_dashboard = worker_dashboard.is_some();
            let mut report = Report {
                outcome: "cancelled",
                overlay_ms: None,
                clipboard_ms: None,
                message: "Capture cancelled.".into(),
            };
            match run(
                &app_worker,
                &sessions,
                mode,
                settle_desktop,
                started,
                &mut report,
            ) {
                Ok(Outcome::Edited(image)) => {
                    if let Some(window) = &worker_dashboard {
                        let _ = window.close();
                    }
                    match editor::open(&app_worker, image, from_dashboard) {
                        Ok(()) => {
                            lease.1 = false;
                            report.outcome = "editing";
                            report.message = "Opened the capture in the editor.".into();
                        }
                        Err(message) => {
                            report.outcome = "error";
                            report.message = message;
                        }
                    }
                }
                Ok(Outcome::Copied) => {
                    report.outcome = "copied";
                    report.message = "Screenshot copied to the clipboard.".into();
                }
                Ok(Outcome::Cancelled) => {}
                Err(message) => {
                    report.outcome = "error";
                    report.message = message;
                }
            }
            app_worker.state::<Diagnostics>().record(report);
            let restore_dashboard = lease.1 && from_dashboard;
            drop(lease);
            notify(&app_worker);
            if restore_dashboard {
                dashboard::request(&app_worker);
            }
        });
    if thread.is_err() {
        sessions.release();
        if let Some(window) = dashboard {
            let _ = window.show();
        }
        return Err("Could not start the capture worker. Try again.".into());
    }
    notify(app);
    Ok(())
}

enum Outcome {
    Cancelled,
    Copied,
    Edited(editor::Image),
}

fn run(
    app: &AppHandle,
    sessions: &Sessions,
    mode: Mode,
    settle_desktop: bool,
    started: Instant,
    report: &mut Report,
) -> Result<Outcome, String> {
    let _dpi = screen::PhysicalDpi::enter()?;
    if settle_desktop {
        unsafe { DwmFlush() };
    }
    if sessions.cancelled.load(Ordering::Acquire) {
        return Ok(Outcome::Cancelled);
    }
    let frame = screen::capture()?;
    match mode {
        Mode::Region => {
            if !sessions.advance(Phase::Selecting) {
                return Ok(Outcome::Cancelled);
            }
            let region = overlay::select(&frame, &sessions.cancelled, started, |elapsed| {
                report.overlay_ms = Some(milliseconds(elapsed));
                notify(app);
            })?;
            let Some(region) = region else {
                return Ok(Outcome::Cancelled);
            };
            if !sessions.advance(Phase::Editing) {
                return Ok(Outcome::Cancelled);
            }
            editor::Image::from_frame(&frame, region).map(Outcome::Edited)
        }
        Mode::Full => {
            if !sessions.advance(Phase::Exporting) {
                return Ok(Outcome::Cancelled);
            }
            let export_started = Instant::now();
            let result = clipboard::copy(&frame, frame.image_rect());
            report.clipboard_ms = Some(milliseconds(export_started.elapsed()));
            result.map(|()| Outcome::Copied)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_rejects_overlap_and_restores_idle_after_cancellation() {
        let sessions = Sessions::new();
        sessions.acquire().unwrap();
        let lease = Lease(&sessions, true);
        assert!(sessions.acquire().is_err());
        assert!(sessions.advance(Phase::Selecting));
        sessions.cancel();
        assert!(!sessions.advance(Phase::Editing));
        drop(lease);
        assert_eq!(sessions.phase(), Phase::Idle);
        sessions.acquire().unwrap();
        assert!(sessions.advance(Phase::Exporting));
        sessions.cancel();
        assert!(!sessions.cancelled.load(Ordering::Acquire));
    }

    #[test]
    fn editing_blocks_new_captures_until_the_editor_closes() {
        let sessions = Sessions::new();
        sessions.acquire().unwrap();
        assert!(sessions.advance(Phase::Selecting));
        assert!(sessions.advance(Phase::Editing));
        drop(Lease(&sessions, false));
        assert!(sessions.acquire().is_err());
        sessions.finish_editing();
        assert!(sessions.acquire().is_ok());
    }
}
