use crate::models::{Note, NoteMetadata, Settings, UpdateNoteRequest, MoveNoteRequest, NoteListResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static MU: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static TEST_DIR: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

fn get_notes_dir() -> PathBuf {
    let test_dir = TEST_DIR.lock().unwrap();
    if let Some(ref path) = *test_dir {
        path.join("notes")
    } else {
        PathBuf::from("notes")
    }
}

fn get_db_file() -> PathBuf {
    let test_dir = TEST_DIR.lock().unwrap();
    if let Some(ref path) = *test_dir {
        path.join("notes_db.json")
    } else {
        PathBuf::from("notes_db.json")
    }
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
    let mut directories: HashMap<String, Vec<String>> = HashMap::new();

    fn walk(dir: &Path, base: &Path, root_files: &mut Vec<String>, dirs: &mut HashMap<String, Vec<String>>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().into_string().map_err(|_| "Invalid filename")?;

            if path.is_dir() {
                if name.starts_with('.') {
                    continue;
                }
                walk(&path, base, root_files, dirs)?;
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let rel_path = path.strip_prefix(base).map_err(|e| e.to_string())?;
                if let Some(parent) = rel_path.parent() {
                    let parent_str = parent.to_str().unwrap_or("");
                    if parent_str.is_empty() {
                        root_files.push(name);
                    } else {
                        dirs.entry(parent_str.to_string()).or_default().push(name);
                    }
                } else {
                    root_files.push(name);
                }
            }
        }
        Ok(())
    }

    walk(&notes_dir, &notes_dir, &mut files, &mut directories)?;

    Ok(NoteListResponse { files, directories })
}

pub fn refresh_notes() -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;
    
    let notes_dir = get_notes_dir();
    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir).map_err(|e| e.to_string())?;
    }

    let mut fs_notes = Vec::new();

    // Helper to scan directory
    fn scan_dir(dir: &Path, base_notes_dir: &Path, fs_notes: &mut Vec<NoteMetadata>) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().into_string().map_err(|_| "Invalid filename")?;

            if path.is_dir() {
                if name.starts_with('.') {
                    continue;
                }
                scan_dir(&path, base_notes_dir, fs_notes)?;
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let rel_path = path.strip_prefix(base_notes_dir).map_err(|e| e.to_string())?;
                let directory = rel_path.parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string();
                
                let filename = rel_path.file_name()
                    .and_then(|s| s.to_str())
                    .ok_or("Invalid filename")?;
                
                let title = filename.strip_suffix(".md").unwrap_or(filename).replace("_", " ");
                
                fs_notes.push(NoteMetadata {
                    argument: title,
                    directory,
                });
            }
        }
        Ok(())
    }

    scan_dir(&notes_dir, &notes_dir, &mut fs_notes)?;

    let (db_notes, settings) = read_db();
    
    // We want to keep DB entries if they still exist on disk, 
    // and add new ones from disk that aren't in the DB.
    // However, the DB might have specific casing for "argument" (title) that the filename (underscored) doesn't preserve perfectly
    // but the current implementation uses title.replace(" ", "_") + ".md" for filename.
    
    let mut final_notes = Vec::new();
    let mut handled_fs_indices = std::collections::HashSet::new();

    // 1. Keep existing DB notes if they exist on disk
    for db_note in db_notes {
        let filename = db_note.argument.replace(" ", "_") + ".md";
        let path = notes_dir.join(&db_note.directory).join(filename);
        
        if path.exists() {
            final_notes.push(db_note.clone());
            
            // Mark this FS note as handled
            if let Some(pos) = fs_notes.iter().position(|f| f.argument == db_note.argument && f.directory == db_note.directory) {
                handled_fs_indices.insert(pos);
            } else if let Some(pos) = fs_notes.iter().position(|f| f.argument.to_lowercase() == db_note.argument.to_lowercase() && f.directory == db_note.directory) {
                // Handle case where casing might have changed but it's the same file
                handled_fs_indices.insert(pos);
            }
        }
    }

    // 2. Add new FS notes that weren't in the DB
    for (i, fs_note) in fs_notes.into_iter().enumerate() {
        if !handled_fs_indices.contains(&i) {
            final_notes.push(fs_note);
        }
    }

    write_db(final_notes, settings)
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
        .find(|n| {
            let n_filename = n.argument.replace(" ", "_");
            (n_filename == filename_no_ext || n.argument == filename_no_ext) && n.directory == directory
        })
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

pub fn rename_note(old_title: &str, directory: &str, new_title: &str) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let old_filename = old_title.replace(" ", "_") + ".md";
    let old_path = get_notes_dir().join(directory).join(&old_filename);

    let new_filename = new_title.replace(" ", "_") + ".md";
    let new_path = get_notes_dir().join(directory).join(&new_filename);

    if !old_path.exists() {
        return Err("Note not found".to_string());
    }

    if new_path.exists() {
        return Err("Target note already exists".to_string());
    }

    fs::rename(old_path, new_path).map_err(|e| e.to_string())?;

    let (mut notes, settings) = read_db();
    for n in notes.iter_mut() {
        if n.argument == old_title && n.directory == directory {
            n.argument = new_title.to_string();
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

pub fn rename_directory(old_name: &str, new_name: &str) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let old_path = get_notes_dir().join(old_name);
    let new_path = get_notes_dir().join(new_name);

    if !old_path.exists() {
        return Err("Directory not found".to_string());
    }

    if new_path.exists() {
        return Err("Target directory already exists".to_string());
    }

    fs::rename(old_path, new_path).map_err(|e| e.to_string())?;

    let (mut notes, settings) = read_db();
    for n in notes.iter_mut() {
        if n.directory == old_name {
            n.directory = new_name.to_string();
        } else if n.directory.starts_with(&format!("{}/", old_name)) {
            n.directory = n.directory.replace(old_name, new_name);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::models::UpdateNoteRequest;

    fn setup_test() -> tempfile::TempDir {
        let dir = tempdir().expect("Failed to create temp dir");
        let mut test_dir = TEST_DIR.lock().unwrap();
        *test_dir = Some(dir.path().to_path_buf());
        dir
    }

    #[test]
    fn test_save_and_list_notes() {
        let _dir = setup_test();
        
        save_note("Hello World", "Test Note", "").expect("Save failed");
        save_note("Sub Note Content", "Sub Note", "drafts").expect("Save failed");

        let notes = list_notes().expect("List failed");
        assert!(notes.files.contains(&"Test_Note.md".to_string()));
        assert!(notes.directories.contains_key("drafts"));
        assert!(notes.directories.get("drafts").unwrap().contains(&"Sub_Note.md".to_string()));
    }

    #[test]
    fn test_get_note() {
        let _dir = setup_test();
        
        save_note("Hello World", "Test Note", "").expect("Save failed");
        
        let note = get_note("Test_Note.md").expect("Get failed");
        assert_eq!(note.title, "Test Note");
        assert_eq!(note.content, "Hello World");
        assert_eq!(note.directory, "");

        save_note("Sub Content", "Sub Note", "folder").expect("Save failed");
        let sub_note = get_note("folder/Sub_Note.md").expect("Get failed");
        assert_eq!(sub_note.title, "Sub Note");
        assert_eq!(sub_note.directory, "folder");
    }

    #[test]
    fn test_update_note() {
        let _dir = setup_test();
        
        save_note("Old Content", "Old Title", "").expect("Save failed");
        
        let req = UpdateNoteRequest {
            old_title: "Old Title".to_string(),
            old_directory: "".to_string(),
            new_title: "New Title".to_string(),
            new_directory: "archived".to_string(),
            new_content: "New Content".to_string(),
        };
        
        update_note(req).expect("Update failed");
        
        let note = get_note("archived/New_Title.md").expect("Get failed");
        assert_eq!(note.title, "New Title");
        assert_eq!(note.content, "New Content");
        assert_eq!(note.directory, "archived");
        
        assert!(get_note("Old_Title.md").is_err());
    }

    #[test]
    fn test_delete_note() {
        let _dir = setup_test();
        
        save_note("Content", "Delete Me", "").expect("Save failed");
        assert!(get_note("Delete_Me.md").is_ok());
        
        delete_note("Delete Me", "").expect("Delete failed");
        assert!(get_note("Delete_Me.md").is_err());
    }

    #[test]
    fn test_nested_directories() {
        let _dir = setup_test();
        
        save_note("Nested Content", "Nested Note", "Informatica/AWS/S3").expect("Save failed");
        
        let notes = list_notes().expect("List failed");
        assert!(notes.directories.contains_key("Informatica/AWS/S3"));
        assert!(notes.directories.get("Informatica/AWS/S3").unwrap().contains(&"Nested_Note.md".to_string()));
    }
}
