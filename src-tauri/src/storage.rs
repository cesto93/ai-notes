use crate::models::{Note, NoteMetadata, Settings, UpdateNoteRequest, MoveNoteRequest, NoteListResponse};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static MU: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn get_notes_dir() -> PathBuf {
    PathBuf::from("notes")
}

fn get_db_file() -> PathBuf {
    PathBuf::from("notes_db.json")
}

#[derive(Debug, Serialize, Deserialize)]
struct TinyDBFile {
    #[serde(rename = "_default", default)]
    default: HashMap<String, NoteMetadata>,
    #[serde(rename = "settings", default)]
    settings: HashMap<String, Settings>,
}

pub fn read_db() -> (Vec<NoteMetadata>, Settings) {
    let db_path = get_db_file();
    if !db_path.exists() {
        return (Vec::new(), Settings { provider: "google".to_string(), model: "gemini-2.0-flash".to_string() });
    }

    let data = fs::read_to_string(db_path).unwrap_or_default();
    let raw: TinyDBFile = serde_json::from_str(&data).unwrap_or(TinyDBFile {
        default: HashMap::new(),
        settings: HashMap::new(),
    });

    let notes: Vec<NoteMetadata> = raw.default.values().cloned().collect();
    // Sort to maintain some order if needed, but TinyDB keys are strings
    
    let settings = raw.settings.get("1").cloned().unwrap_or(Settings {
        provider: "google".to_string(),
        model: "gemini-2.0-flash".to_string(),
    });

    (notes, settings)
}

pub fn write_db(notes: Vec<NoteMetadata>, settings: Settings) -> Result<(), String> {
    let mut default_table = HashMap::new();
    for (i, note) in notes.into_iter().enumerate() {
        default_table.insert((i + 1).to_string(), note);
    }

    let mut settings_table = HashMap::new();
    settings_table.insert("1".to_string(), settings);

    let raw = TinyDBFile {
        default: default_table,
        settings: settings_table,
    };

    let data = serde_json::to_string_pretty(&raw).map_err(|e| e.to_string())?;
    fs::write(get_db_file(), data).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn create_directory(directory: &str) -> Result<(), String> {
    let path = get_notes_dir().join(directory);
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub fn save_note(content: &str, title: &str, directory: &str) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let target_dir = if directory.is_empty() {
        get_notes_dir()
    } else {
        get_notes_dir().join(directory)
    };

    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    let filename = title.replace(" ", "_") + ".md";
    let note_path = target_dir.join(filename);
    fs::write(note_path, content).map_err(|e| e.to_string())?;

    let (mut notes, settings) = read_db();
    notes.push(NoteMetadata {
        argument: title.to_string(),
        directory: directory.to_string(),
    });
    write_db(notes, settings)
}

pub fn list_notes() -> Result<NoteListResponse, String> {
    let notes_dir = get_notes_dir();
    if !notes_dir.exists() {
        return Ok(NoteListResponse {
            files: Vec::new(),
            directories: HashMap::new(),
        });
    }

    let mut files = Vec::new();
    let mut directories = HashMap::new();

    let entries = fs::read_dir(notes_dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().into_string().map_err(|_| "Invalid filename")?;

        if path.is_dir() {
            if name.starts_with('.') {
                continue;
            }
            let mut sub_notes = Vec::new();
            let sub_entries = fs::read_dir(&path).map_err(|e| e.to_string())?;
            for sub_entry in sub_entries {
                let sub_entry = sub_entry.map_err(|e| e.to_string())?;
                let sub_path = sub_entry.path();
                if sub_path.is_file() && sub_path.extension().and_then(|s| s.to_str()) == Some("md") {
                    sub_notes.push(sub_entry.file_name().into_string().map_err(|_| "Invalid filename")?);
                }
            }
            directories.insert(name, sub_notes);
        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            files.push(name);
        }
    }

    Ok(NoteListResponse { files, directories })
}

pub fn get_note(path_str: &str) -> Result<Note, String> {
    let full_path = get_notes_dir().join(path_str);
    if !full_path.exists() {
        return Err("Note not found".to_string());
    }

    let content = fs::read_to_string(&full_path).map_err(|e| e.to_string())?;
    
    let path = Path::new(path_str);
    let directory = path.parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();
    
    let filename = path.file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid filename")?;
    
    let filename_no_ext = filename.strip_suffix(".md").unwrap_or(filename);
    
    let (notes, _) = read_db();
    let title = notes.iter()
        .find(|n| n.argument.replace(" ", "_") == filename_no_ext && n.directory == directory)
        .map(|n| n.argument.clone())
        .unwrap_or_else(|| filename_no_ext.to_string());

    Ok(Note {
        content,
        title,
        directory,
    })
}

pub fn update_note(req: UpdateNoteRequest) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let old_filename = req.old_title.replace(" ", "_") + ".md";
    let old_path = get_notes_dir().join(&req.old_directory).join(&old_filename);

    let new_target_dir = get_notes_dir().join(&req.new_directory);
    fs::create_dir_all(&new_target_dir).map_err(|e| e.to_string())?;

    let new_filename = req.new_title.replace(" ", "_") + ".md";
    let new_path = new_target_dir.join(&new_filename);

    if old_path != new_path && old_path.exists() {
        fs::remove_file(old_path).map_err(|e| e.to_string())?;
    }

    fs::write(new_path, &req.new_content).map_err(|e| e.to_string())?;

    let (mut notes, settings) = read_db();
    for n in notes.iter_mut() {
        if n.argument == req.old_title && n.directory == req.old_directory {
            n.argument = req.new_title.clone();
            n.directory = req.new_directory.clone();
            break;
        }
    }
    write_db(notes, settings)
}

pub fn move_note(req: MoveNoteRequest) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let filename_no_ext = req.title.replace(" ", "_");
    let (mut notes, settings) = read_db();

    let mut actual_title = req.title.clone();
    for n in notes.iter() {
        if n.argument.replace(" ", "_") == filename_no_ext && n.directory == req.old_directory {
            actual_title = n.argument.clone();
            break;
        }
    }

    for n in notes.iter_mut() {
        if n.argument == actual_title && n.directory == req.old_directory {
            n.directory = req.new_directory.clone();
            break;
        }
    }
    write_db(notes, settings)?;

    let filename = format!("{}.md", filename_no_ext);
    let old_path = get_notes_dir().join(&req.old_directory).join(&filename);
    
    let new_target_dir = get_notes_dir().join(&req.new_directory);
    fs::create_dir_all(&new_target_dir).map_err(|e| e.to_string())?;
    
    let new_path = new_target_dir.join(&filename);

    if old_path.exists() {
        fs::rename(old_path, new_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn delete_note(title: &str, directory: &str) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let filename_no_ext = title.replace(" ", "_");
    let filename = format!("{}.md", filename_no_ext);
    let path = get_notes_dir().join(directory).join(filename);

    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    let (mut notes, settings) = read_db();
    notes.retain(|n| {
        !( (n.argument.replace(" ", "_") == filename_no_ext && n.directory == directory) || 
           (n.argument == title && n.directory == directory) )
    });
    write_db(notes, settings)
}

pub fn delete_directory(directory: &str) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let path = get_notes_dir().join(directory);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }

    let (mut notes, settings) = read_db();
    notes.retain(|n| n.directory != directory);
    write_db(notes, settings)
}

pub fn get_settings() -> Result<Settings, String> {
    let (_, settings) = read_db();
    Ok(settings)
}

pub fn save_settings(provider: String, model: String) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;
    let (notes, _) = read_db();
    write_db(notes, Settings { provider, model })
}

use serde::{Deserialize, Serialize};
