use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub outcome: &'static str,
    pub overlay_ms: Option<f64>,
    pub clipboard_ms: Option<f64>,
    pub message: String,
}

pub struct Diagnostics {
    started: Instant,
    ready: std::sync::Once,
    last: Mutex<Option<Report>>,
    // Opt-in for local benchmarking. Only outcomes and timings are written.
    output: Mutex<Option<PathBuf>>,
}

pub fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

impl Diagnostics {
    pub fn new(started: Instant) -> Self {
        Self {
            started,
            ready: std::sync::Once::new(),
            last: Mutex::new(None),
            output: Mutex::new(std::env::var_os("QAPTURE_DIAGNOSTICS").map(PathBuf::from)),
        }
    }

    fn log(&self, event: &impl Serialize) {
        let output = self.output.lock().unwrap();
        let Some(path) = output.as_ref() else { return };
        let Ok(line) = serde_json::to_string(event) else {
            return;
        };
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{line}");
        }
    }

    pub fn dashboard_ready(&self) {
        self.ready.call_once(|| {
            self.log(&serde_json::json!({
                "event": "dashboard-ready",
                "startupMs": milliseconds(self.started.elapsed()),
            }))
        });
    }

    pub fn record(&self, report: Report) {
        self.log(&serde_json::json!({
            "event": "capture-finished",
            "outcome": report.outcome,
            "overlayMs": report.overlay_ms,
            "clipboardMs": report.clipboard_ms,
        }));
        *self.last.lock().unwrap() = Some(report);
    }

    pub fn latest(&self) -> Option<Report> {
        self.last.lock().unwrap().clone()
    }
}
