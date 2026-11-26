use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::Error as RegistrationError;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::{
    session::{self, Mode},
    settings::{Settings, Store},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutStatus {
    pub settings: Settings,
    pub region_active: bool,
    pub full_active: bool,
    pub warning: Option<String>,
    pub region_error: Option<String>,
    pub full_error: Option<String>,
}

pub struct Shortcuts {
    store: Store,
    status: Mutex<ShortcutStatus>,
    region_id: AtomicU32,
    full_id: AtomicU32,
    recording: AtomicBool,
}

impl Shortcuts {
    pub fn new(store: Store) -> Self {
        let (settings, warning) = store.load();
        Self {
            store,
            status: Mutex::new(ShortcutStatus {
                settings,
                region_active: false,
                full_active: false,
                warning,
                region_error: None,
                full_error: None,
            }),
            region_id: AtomicU32::new(0),
            full_id: AtomicU32::new(0),
            recording: AtomicBool::new(false),
        }
    }

    pub fn status(&self) -> ShortcutStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_recording(&self, recording: bool) {
        self.recording.store(recording, Ordering::Release);
    }

    fn id(&self, mode: Mode) -> &AtomicU32 {
        match mode {
            Mode::Region => &self.region_id,
            Mode::Full => &self.full_id,
        }
    }

    fn register(app: &AppHandle, mode: Mode, shortcut: Shortcut) -> Result<(), RegistrationError> {
        app.global_shortcut()
            .on_shortcut(shortcut, move |app, key, event| {
                let shortcuts = app.state::<Shortcuts>();
                if event.state() == ShortcutState::Pressed
                    && shortcuts.id(mode).load(Ordering::Acquire) == key.id()
                {
                    if shortcuts.recording.load(Ordering::Acquire)
                        && app
                            .get_webview_window("main")
                            .is_some_and(|window| window.is_focused().unwrap_or(false))
                    {
                        // Windows routes already-registered keys to us before the WebView sees them.
                        let _ =
                            app.emit_to("main", "qapture-shortcut-recorded", recorded_binding(key));
                    } else {
                        session::start_background(app, mode, false);
                    }
                }
            })
    }

    pub fn initialize(&self, app: &AppHandle) {
        let mut status = self.status.lock().unwrap();
        for mode in [Mode::Region, Mode::Full] {
            let result = parse(status.settings.get(mode)).and_then(|shortcut| {
                Self::register(app, mode, shortcut).map_err(registration_message)?;
                Ok(shortcut)
            });
            match result {
                Ok(shortcut) => {
                    self.id(mode).store(shortcut.id(), Ordering::Release);
                    set_active(&mut status, mode, None);
                }
                Err(error) => set_active(&mut status, mode, Some(error)),
            }
        }
    }

    pub fn update(
        &self,
        app: &AppHandle,
        mode: Mode,
        binding: String,
    ) -> Result<ShortcutStatus, String> {
        let candidate = parse(&binding)?;
        let mut status = self.status.lock().unwrap();
        let other = match mode {
            Mode::Region => Mode::Full,
            Mode::Full => Mode::Region,
        };
        if parse(status.settings.get(other)).is_ok_and(|shortcut| shortcut == candidate) {
            return Err("The other capture action already uses this shortcut.".into());
        }
        let id = self.id(mode);
        let previous = parse(status.settings.get(mode))
            .ok()
            .filter(|shortcut| shortcut.id() == id.load(Ordering::Acquire));
        if previous == Some(candidate) {
            return Ok(status.clone());
        }
        Self::register(app, mode, candidate).map_err(registration_message)?;
        self.commit_binding(app, &mut status, mode, binding, candidate, previous)
    }

    pub fn recover(&self, app: &AppHandle, mode: Mode) -> Result<ShortcutStatus, String> {
        let mut status = self.status.lock().unwrap();
        if self.id(mode).load(Ordering::Acquire) != 0 {
            return Ok(status.clone());
        }
        let other = match mode {
            Mode::Region => Mode::Full,
            Mode::Full => Mode::Region,
        };
        let (binding, candidate) = reserve_available(
            mode,
            status.settings.get(mode),
            parse(status.settings.get(other)).ok(),
            |shortcut| Self::register(app, mode, shortcut),
        )?;
        self.commit_binding(app, &mut status, mode, binding, candidate, None)
    }

    fn commit_binding(
        &self,
        app: &AppHandle,
        status: &mut ShortcutStatus,
        mode: Mode,
        binding: String,
        candidate: Shortcut,
        previous: Option<Shortcut>,
    ) -> Result<ShortcutStatus, String> {
        let mut next = status.settings.clone();
        next.set(mode, binding);
        if next.get(mode) != status.settings.get(mode)
            && let Err(error) = self.store.save(&next)
        {
            let _ = app.global_shortcut().unregister(candidate);
            return Err(error);
        }
        self.id(mode).store(candidate.id(), Ordering::Release);
        status.settings = next;
        status.warning = None;
        set_active(status, mode, None);
        if let Some(previous) = previous
            && app.global_shortcut().unregister(previous).is_err()
        {
            status.warning =
                Some("Restart Qapture to release the previous shortcut reservation.".into());
        }
        Ok(status.clone())
    }
}

fn already_in_use(error: &RegistrationError) -> bool {
    // The plugin converts the native error to a string before returning it.
    matches!(error, RegistrationError::GlobalHotkey(message) if message.starts_with("HotKey already registered:"))
}

fn registration_message(error: RegistrationError) -> String {
    if already_in_use(&error) {
        "Shortcut already in use.".into()
    } else {
        "Windows could not activate this shortcut.".into()
    }
}

fn reserve_available(
    mode: Mode,
    saved: &str,
    other: Option<Shortcut>,
    mut register: impl FnMut(Shortcut) -> Result<(), RegistrationError>,
) -> Result<(String, Shortcut), String> {
    let alternatives = match mode {
        Mode::Region => ["Ctrl+Alt+F9", "Ctrl+Shift+F7", "Ctrl+Alt+Shift+F9"],
        Mode::Full => ["Ctrl+Alt+F10", "Ctrl+Shift+F8", "Ctrl+Alt+Shift+F10"],
    };
    let preferred = parse(saved).ok();
    for binding in std::iter::once(saved).chain(
        alternatives
            .into_iter()
            .filter(|binding| parse(binding).ok() != preferred),
    ) {
        let Ok(candidate) = parse(binding) else {
            continue;
        };
        if Some(candidate) == other {
            continue;
        }
        match register(candidate) {
            Ok(()) => return Ok((binding.to_owned(), candidate)),
            Err(error) if already_in_use(&error) => continue,
            Err(error) => return Err(registration_message(error)),
        }
    }
    Err("Suggested shortcuts are busy. Choose Edit to record another combination.".into())
}

fn recorded_binding(shortcut: &Shortcut) -> String {
    let key = shortcut.key.to_string();
    [
        (Modifiers::CONTROL, "Ctrl"),
        (Modifiers::ALT, "Alt"),
        (Modifiers::SHIFT, "Shift"),
        (Modifiers::SUPER, "Super"),
    ]
    .into_iter()
    .filter_map(|(modifier, label)| shortcut.mods.contains(modifier).then_some(label))
    .chain(std::iter::once(key.as_str()))
    .collect::<Vec<_>>()
    .join("+")
}

fn set_active(status: &mut ShortcutStatus, mode: Mode, error: Option<String>) {
    match mode {
        Mode::Region => {
            status.region_active = error.is_none();
            status.region_error = error;
        }
        Mode::Full => {
            status.full_active = error.is_none();
            status.full_error = error;
        }
    }
}

fn parse(binding: &str) -> Result<Shortcut, String> {
    if binding.len() > 64 {
        return Err("The shortcut is too long.".into());
    }
    let key = binding
        .parse::<Shortcut>()
        .map_err(|_| "Choose a supported Windows shortcut.")?;
    if !key.mods.intersects(Modifiers::CONTROL | Modifiers::ALT) {
        return Err("Include Ctrl or Alt in the shortcut.".into());
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_uses_the_saved_chord_if_it_is_available_again() {
        let result = reserve_available(Mode::Region, "Alt+KeyZ", None, |_| Ok(())).unwrap();
        assert_eq!(result.0, "Alt+KeyZ");
    }

    #[test]
    fn recovery_skips_conflicts_duplicates_and_the_other_actions_saved_chord() {
        let mut tried = Vec::new();
        let result = reserve_available(
            Mode::Region,
            "Alt+Control+F9",
            parse("Ctrl+Shift+F7").ok(),
            |key| {
                tried.push(key);
                if tried.len() == 1 {
                    Err(RegistrationError::GlobalHotkey(
                        "HotKey already registered: fixture".into(),
                    ))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap();
        assert_eq!(tried.len(), 2);
        assert_eq!(result.0, "Ctrl+Alt+Shift+F9");
    }

    #[test]
    fn recovery_stops_on_other_windows_errors_and_reports_exhaustion() {
        let mut calls = 0;
        let error = reserve_available(Mode::Full, "Ctrl+Shift+F10", None, |_| {
            calls += 1;
            Err(RegistrationError::GlobalHotkey(
                "other Windows error".into(),
            ))
        })
        .unwrap_err();
        assert_eq!(calls, 1);
        assert_eq!(error, "Windows could not activate this shortcut.");
        let error = reserve_available(Mode::Full, "invalid", None, |_| {
            Err(RegistrationError::GlobalHotkey(
                "HotKey already registered: fixture".into(),
            ))
        })
        .unwrap_err();
        assert!(error.contains("Choose Edit"));
    }

    #[test]
    fn validates_modifiers_and_normalizes_shortcut_identity() {
        assert!(parse("F9").is_err());
        assert!(parse("Shift+F9").is_err());
        assert!(parse("Ctrl+Shift+").is_err());
        assert!(parse("Ctrl+KeyQ").is_ok());
        assert_eq!(parse("ctrl+shift+F9"), parse("Shift+Control+F9"));
        assert_eq!(
            recorded_binding(&parse("Shift+Control+F9").unwrap()),
            "Ctrl+Shift+F9"
        );
    }
}
