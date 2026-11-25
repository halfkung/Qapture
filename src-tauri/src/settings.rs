use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::session::Mode;

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub region: String,
    pub full: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            region: "Ctrl+Shift+F9".into(),
            full: "Ctrl+Shift+F10".into(),
        }
    }
}

impl Settings {
    pub fn get(&self, mode: Mode) -> &str {
        match mode {
            Mode::Region => &self.region,
            Mode::Full => &self.full,
        }
    }

    pub fn set(&mut self, mode: Mode, value: String) {
        match mode {
            Mode::Region => self.region = value,
            Mode::Full => self.full = value,
        }
    }
}

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn new(directory: &Path) -> Self {
        Self {
            path: directory.join("settings.json"),
        }
    }

    pub fn load(&self) -> (Settings, Option<String>) {
        match fs::read(&self.path) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(settings) => (settings, None),
                Err(_) => (
                    Settings::default(),
                    Some("Settings were unreadable. Default shortcuts are in use.".into()),
                ),
            },
            Err(error) if error.kind() == ErrorKind::NotFound => (Settings::default(), None),
            Err(error) => (
                Settings::default(),
                Some(format!("Could not read shortcut settings ({error}).")),
            ),
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), String> {
        let write = || -> std::io::Result<()> {
            if let Some(directory) = self.path.parent() {
                fs::create_dir_all(directory)?;
            }
            let pending = self.path.with_extension("tmp");
            fs::write(&pending, serde_json::to_vec_pretty(settings)?)?;
            if let Err(error) = fs::rename(&pending, &self.path) {
                let _ = fs::remove_file(&pending);
                return Err(error);
            }
            Ok(())
        };
        write().map_err(|error| format!("Could not save shortcut settings ({error})."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn settings_load_defaults_and_replace_an_existing_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("qapture-settings-{}-{unique}", std::process::id()));
        let store = Store::new(&root);
        let (mut settings, warning) = store.load();
        assert!(warning.is_none());
        assert_eq!(settings.region, "Ctrl+Shift+F9");
        store.save(&settings).unwrap();
        settings.full = "Ctrl+Alt+KeyP".into();
        store.save(&settings).unwrap();
        assert_eq!(store.load().0.full, "Ctrl+Alt+KeyP");
        fs::write(&store.path, b"invalid").unwrap();
        let (settings, warning) = store.load();
        assert_eq!(settings.full, "Ctrl+Shift+F10");
        assert!(warning.is_some());
        fs::remove_dir_all(root).unwrap();
    }
}
