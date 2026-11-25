use std::sync::{
    Mutex,
    atomic::{AtomicU32, Ordering},
};

use serde::Serialize;
use tauri::{AppHandle, Manager};
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
        }
    }

    pub fn status(&self) -> ShortcutStatus {
        self.status.lock().unwrap().clone()
    }

    fn id(&self, mode: Mode) -> &AtomicU32 {
        match mode {
            Mode::Region => &self.region_id,
            Mode::Full => &self.full_id,
        }
    }

    fn register(app: &AppHandle, mode: Mode, shortcut: Shortcut) -> Result<(), String> {
        app.global_shortcut()
            .on_shortcut(shortcut, move |app, key, event| {
                if event.state() == ShortcutState::Pressed
                    && app.state::<Shortcuts>().id(mode).load(Ordering::Acquire) == key.id()
                {
                    session::start_background(app, mode, false);
                }
            })
            .map_err(|error| {
                format!(
                    "Windows could not register the shortcut. It may already be in use. ({error})"
                )
            })
    }

    pub fn initialize(&self, app: &AppHandle) {
        let mut status = self.status.lock().unwrap();
        for mode in [Mode::Region, Mode::Full] {
            let result = parse(status.settings.get(mode)).and_then(|shortcut| {
                Self::register(app, mode, shortcut)?;
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
        Self::register(app, mode, candidate)?;
        let mut next = status.settings.clone();
        next.set(mode, binding);
        if let Err(error) = self.store.save(&next) {
            let _ = app.global_shortcut().unregister(candidate);
            return Err(error);
        }
        id.store(candidate.id(), Ordering::Release);
        status.settings = next;
        status.warning = None;
        set_active(&mut status, mode, None);
        if let Some(previous) = previous
            && app.global_shortcut().unregister(previous).is_err()
        {
            status.warning =
                Some("Restart Qapture to release the previous shortcut reservation.".into());
        }
        Ok(status.clone())
    }
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
    fn validates_modifiers_and_normalizes_shortcut_identity() {
        assert!(parse("F9").is_err());
        assert!(parse("Shift+F9").is_err());
        assert!(parse("Ctrl+Shift+").is_err());
        assert!(parse("Ctrl+KeyQ").is_ok());
        assert_eq!(parse("ctrl+shift+F9"), parse("Shift+Control+F9"));
    }
}
