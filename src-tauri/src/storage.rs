use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Manager, State,
    ipc::{InvokeBody, Request},
};

use crate::{
    editor::{Editor, Image},
    session::Sessions,
};

const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
const MAX_IMAGE_BYTES: u64 = 8 + 64 * 1024 * 1024 * 4;
const MAGIC: &[u8] = b"QAP1";

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub retain_projects: bool,
    pub auto_save: bool,
    pub directory: String,
    pub prefix: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            retain_projects: false,
            auto_save: false,
            directory: String::new(),
            prefix: "Qapture".into(),
        }
    }
}

pub struct Storage {
    root: PathBuf,
    preferences: Mutex<Preferences>,
    files: Mutex<()>,
}

impl Storage {
    pub fn new(root: PathBuf) -> Self {
        let preferences = fs::read(root.join("output.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self {
            root,
            preferences: Mutex::new(preferences),
            files: Mutex::new(()),
        }
    }

    pub fn preferences(&self) -> Preferences {
        self.preferences.lock().unwrap().clone()
    }

    fn path(&self, id: &str) -> Result<PathBuf, String> {
        if id.is_empty() || id.len() > 24 || !id.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("Invalid history item.".into());
        }
        Ok(self.root.join("projects").join(format!("{id}.qapture")))
    }

    fn keep(&self, title: &str, document: serde_json::Value, bytes: &[u8]) -> Result<(), String> {
        let preference = self.preferences.lock().unwrap();
        if !preference.retain_projects {
            return Err("Enable editable project retention in History first.".into());
        }
        let _files = self.files.lock().unwrap();
        let header = serde_json::to_vec(&ProjectHeader {
            title: title.trim().chars().take(100).collect(),
            document,
        })
        .map_err(problem)?;
        if header.len() > MAX_DOCUMENT_BYTES {
            return Err("The annotation document is too large to retain.".into());
        }
        let path = self.path(&stamp())?;
        fs::create_dir_all(path.parent().unwrap()).map_err(problem)?;
        let pending = path.with_extension("tmp");
        let result = (|| -> std::io::Result<()> {
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&pending)?;
            output.write_all(MAGIC)?;
            output.write_all(&(header.len() as u32).to_le_bytes())?;
            output.write_all(&header)?;
            output.write_all(bytes)?;
            output.sync_all()?;
            drop(output);
            fs::rename(&pending, &path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(pending);
        }
        result.map_err(problem)
    }

    fn items(&self) -> Result<Vec<ProjectItem>, String> {
        let entries = match fs::read_dir(self.root.join("projects")) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(error) => return Err(problem(error)),
        };
        let mut items = Vec::new();
        for entry in entries {
            let path = entry.map_err(problem)?.path();
            let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if self.path(id).ok().as_ref() != Some(&path) || !path.is_file() {
                continue;
            }
            let title = File::open(&path)
                .ok()
                .and_then(|mut file| read_header(&mut file).ok())
                .map_or_else(|| "Unreadable project".into(), |header| header.title);
            items.push(ProjectItem {
                id: id.into(),
                title,
            });
        }
        items.sort_unstable_by(|a, b| b.id.cmp(&a.id));
        Ok(items)
    }
}

#[derive(Serialize)]
pub struct ProjectItem {
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize)]
struct ProjectHeader {
    title: String,
    document: serde_json::Value,
}

fn read_header(file: &mut File) -> Result<ProjectHeader, String> {
    let mut prefix = [0; 8];
    file.read_exact(&mut prefix).map_err(problem)?;
    if &prefix[..4] != MAGIC {
        return Err("This is not a Qapture project.".into());
    }
    let length = u32::from_le_bytes(prefix[4..].try_into().unwrap()) as usize;
    if length > MAX_DOCUMENT_BYTES {
        return Err("Project metadata is too large.".into());
    }
    let mut header = vec![0; length];
    file.read_exact(&mut header).map_err(problem)?;
    serde_json::from_slice(&header).map_err(problem)
}

pub fn stamp() -> String {
    format!(
        "{:024}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn problem(error: impl std::fmt::Display) -> String {
    format!("Could not access local Qapture storage. {error}")
}

#[tauri::command]
pub fn output_preferences(storage: State<'_, Storage>) -> Preferences {
    storage.preferences()
}

#[tauri::command]
pub async fn set_output_preferences(
    storage: State<'_, Storage>,
    preferences: Preferences,
) -> Result<Preferences, String> {
    let directory = Path::new(&preferences.directory);
    if (preferences.auto_save || !preferences.directory.is_empty())
        && (!directory.is_absolute() || !directory.is_dir())
    {
        return Err("Enter the full path to an existing folder, or clear the folder and turn off Auto Save.".into());
    }
    if preferences.prefix.is_empty()
        || preferences.prefix.chars().count() > 48
        || preferences
            .prefix
            .chars()
            .any(|ch| ch.is_control() || "<>:\"/\\|?*".contains(ch))
        || preferences.prefix.ends_with(['.', ' '])
    {
        return Err("Enter a file name prefix without Windows filename punctuation.".into());
    }
    let mut current = storage.preferences.lock().unwrap();
    fs::create_dir_all(&storage.root).map_err(problem)?;
    let pending = storage.root.join("output.tmp");
    fs::write(
        &pending,
        serde_json::to_vec_pretty(&preferences).map_err(problem)?,
    )
    .map_err(problem)?;
    fs::rename(&pending, storage.root.join("output.json")).map_err(problem)?;
    *current = preferences.clone();
    Ok(preferences)
}

#[tauri::command]
pub async fn list_projects(storage: State<'_, Storage>) -> Result<Vec<ProjectItem>, String> {
    let _files = storage.files.lock().unwrap();
    storage.items()
}

#[tauri::command]
pub async fn keep_project(
    storage: State<'_, Storage>,
    editor: State<'_, Editor>,
    title: String,
    document: serde_json::Value,
) -> Result<(), String> {
    editor.with_original(|bytes| storage.keep(&title, document, bytes))
}

#[tauri::command]
pub async fn delete_project(storage: State<'_, Storage>, id: String) -> Result<(), String> {
    let _files = storage.files.lock().unwrap();
    fs::remove_file(storage.path(&id)?).map_err(problem)
}

#[tauri::command]
pub async fn clear_projects(storage: State<'_, Storage>) -> Result<(), String> {
    let _files = storage.files.lock().unwrap();
    for item in storage.items()? {
        fs::remove_file(storage.path(&item.id)?).map_err(problem)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn reopen_project(
    app: AppHandle,
    storage: State<'_, Storage>,
    id: String,
) -> Result<(), String> {
    let (image, document) = {
        let _files = storage.files.lock().unwrap();
        let mut file = File::open(storage.path(&id)?).map_err(problem)?;
        let header = read_header(&mut file)?;
        let mut packet = Vec::new();
        file.take(MAX_IMAGE_BYTES + 1)
            .read_to_end(&mut packet)
            .map_err(problem)?;
        (Image::from_packet(packet)?, header.document)
    };
    app.state::<Sessions>()
        .open_editor(&app, image, Some(document))
}

#[tauri::command]
pub async fn import_image(app: AppHandle, request: Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(packet) = request.body() else {
        return Err("The imported pixels are missing.".into());
    };
    if packet.len() as u64 > MAX_IMAGE_BYTES {
        return Err("The imported image is too large.".into());
    }
    let image = Image::from_packet(packet.clone())?;
    app.state::<Sessions>().open_editor(&app, image, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_is_opt_in_and_project_files_keep_the_original_separate() {
        let directory = std::env::temp_dir().join(format!("qapture-fixture-{}", stamp()));
        let storage = Storage::new(directory.clone());
        let packet = [1, 0, 0, 0, 1, 0, 0, 0, 1, 2, 3, 255];
        let document = serde_json::json!({"marks": []});
        assert!(storage.keep("Fixture", document.clone(), &packet).is_err());
        assert!(!directory.exists());
        assert!(storage.path("../output").is_err());
        storage.preferences.lock().unwrap().retain_projects = true;
        storage.keep("ภาษาไทย", document.clone(), &packet).unwrap();
        let items = storage.items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "ภาษาไทย");
        let path = storage.path(&items[0].id).unwrap();
        let mut file = File::open(&path).unwrap();
        assert_eq!(read_header(&mut file).unwrap().document, document);
        let mut restored = Vec::new();
        file.read_to_end(&mut restored).unwrap();
        assert_eq!(restored, packet);
        drop(file);
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory.join("projects")).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
