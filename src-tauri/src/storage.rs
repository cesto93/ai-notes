use crate::models::{Note, NoteMetadata, Settings, UpdateNoteRequest, MoveNoteRequest, NoteListResponse, SearchResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static MU: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static TEST_DIR: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

fn default_settings() -> Settings {
    Settings {
        provider: "google".to_string(),
        model: "gemini-2.0-flash".to_string(),
        notes_dir: String::new(),
    }
}

/// Expand a leading `~` to the user's home directory (Linux dev machine convention).
fn expand_home(path: &str) -> String {
    if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    } else if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home, rest);
        }
    }
    path.to_string()
}

fn get_notes_dir() -> Result<PathBuf, String> {
    // Tests always use their sandbox, regardless of settings.
    let test_dir = TEST_DIR.lock().unwrap();
    if let Some(ref path) = *test_dir {
        return Ok(path.join("notes"));
    }
    drop(test_dir);

    // Root folder configured in Settings. There is no default: if unset,
    // the app asks the user to pick one instead of falling back silently.
    let notes_dir = read_db().1.notes_dir;
    let trimmed = notes_dir.trim();
    if trimmed.is_empty() {
        Err("Notes folder is not configured. Please choose one in the app.".to_string())
    } else {
        Ok(PathBuf::from(expand_home(trimmed)))
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
        return (Vec::new(), default_settings());
    }

    let data = fs::read_to_string(db_path).unwrap_or_default();
    let raw: TinyDBFile = serde_json::from_str(&data).unwrap_or(TinyDBFile {
        default: HashMap::new(),
        settings: HashMap::new(),
    });

    let notes: Vec<NoteMetadata> = raw.default.values().cloned().collect();
    // Sort to maintain some order if needed, but TinyDB keys are strings
    
    let settings = raw.settings.get("1").cloned().unwrap_or_else(default_settings);

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
    let path = get_notes_dir()?.join(directory);
    fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub fn save_note(content: &str, title: &str, directory: &str) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let notes_root = get_notes_dir()?;
    let target_dir = if directory.is_empty() {
        notes_root
    } else {
        notes_root.join(directory)
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
    let notes_dir = get_notes_dir()?;
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
                let rel_path = path.strip_prefix(base).map_err(|e| e.to_string())?;
                let rel_path_str = rel_path.to_str().unwrap_or("");
                if !rel_path_str.is_empty() {
                    dirs.entry(rel_path_str.to_string()).or_default();
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

/// Extract inline `#hashtags` from note content. Markdown headings (`# Title`,
/// `## Sub`) are not tags — a tag requires a word character right after `#`
/// and a whitespace/start-of-line before it.
fn extract_tags(content: &str) -> Vec<String> {
    let mut tags = std::collections::HashSet::new();
    let bytes = content.as_bytes();
    for (i, c) in content.char_indices() {
        if c != '#' {
            continue;
        }
        if i > 0 && !bytes[i - 1].is_ascii_whitespace() {
            continue;
        }
        let next = match bytes.get(i + 1) {
            Some(&b) => b,
            None => continue,
        };
        if !(next.is_ascii_alphanumeric() || next == b'_') {
            continue; // plain "## heading" or trailing "#"
        }
        let rest = &content[i + 1..];
        let raw: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && !matches!(*c, '#' | '/' | '.'))
            .collect();
        let tag = raw.trim_matches(|c: char| matches!(c, '-' | ',' | ':' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '!' | '?' | '"' | '\''));
        if !tag.is_empty() {
            tags.insert(tag.to_string());
        }
    }
    tags.into_iter().collect()
}

/// Case-insensitive search across note titles, inline `#hashtags` and content.
/// Results are ranked: title matches first, then tag matches, then content-only.
pub fn search_notes(query: &str) -> Result<Vec<SearchResult>, String> {
    let notes_dir = get_notes_dir()?;
    let query_lower = query.trim().to_lowercase();
    if query_lower.is_empty() || !notes_dir.exists() {
        return Ok(Vec::new());
    }

    let (db_notes, _) = read_db();
    let mut results: Vec<SearchResult> = Vec::new();

    fn walk_search(
        dir: &Path,
        base: &Path,
        query_lower: &str,
        db_notes: &[NoteMetadata],
        results: &mut Vec<SearchResult>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                if entry.file_name().to_string_lossy().starts_with('.') {
                    continue;
                }
                walk_search(&path, base, query_lower, db_notes, results)?;
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let rel_path = match path.strip_prefix(base) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let filename = match path.file_name().and_then(|s| s.to_str()) {
                    Some(f) => f,
                    None => continue,
                };
                let filename_no_ext = filename.strip_suffix(".md").unwrap_or(filename);
                let display_title = filename_no_ext.replace('_', " ");
                let directory = rel_path
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string();
                // DB may hold the original casing for the title; fall back to filename.
                let title = db_notes
                    .iter()
                    .find(|n| {
                        n.argument.replace(" ", "_") == filename_no_ext
                            && n.directory == directory
                    })
                    .map(|n| n.argument.clone())
                    .unwrap_or_else(|| display_title.clone());

                // Skip files that can't be read (invalid UTF-8, races) instead of
                // failing the whole search.
                let content = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let content_lower = content.to_lowercase();
                let title_lower = title.to_lowercase();

                let match_title = title_lower.contains(query_lower);
                let tags = extract_tags(&content);
                let matched_tags: Vec<String> = tags
                    .iter()
                    .filter(|t| t.to_lowercase().contains(query_lower))
                    .cloned()
                    .collect();
                let match_tags = !matched_tags.is_empty();
                let match_content = content_lower.contains(query_lower);

                if !(match_title || match_tags || match_content) {
                    continue;
                }

                // Snippet: the line containing the first content match, centered
                // on the match if too long; otherwise the first plain line.
                let snippet = if let Some(idx) = content_lower.find(&query_lower) {
                    let line_start =
                        content[..idx].rfind('\n').map(|p| p + 1).unwrap_or(0);
                    let line_end = content[idx..]
                        .find('\n')
                        .map(|p| idx + p)
                        .unwrap_or(content.len());
                    let line = content[line_start..line_end].trim();
                    if line.chars().count() > 160 {
                        let match_chars = content[line_start..idx].chars().count();
                        let start = match_chars.saturating_sub(60);
                        line.chars().skip(start).take(160).collect::<String>().trim().to_string()
                    } else {
                        line.to_string()
                    }
                } else {
                    content
                        .lines()
                        .map(str::trim)
                        .find(|l| !l.is_empty() && !l.starts_with('#'))
                        .map(|l| l.chars().take(160).collect())
                        .unwrap_or_default()
                };

                results.push(SearchResult {
                    path: rel_path.to_string_lossy().into_owned(),
                    title,
                    directory,
                    snippet,
                    tags: matched_tags,
                    match_title,
                    match_tags,
                    match_content,
                });
            }
        }
        Ok(())
    }

    walk_search(&notes_dir, &notes_dir, &query_lower, &db_notes, &mut results)?;

    // Rank: title matches first, then tag matches; stable keeps walk order.
    results.sort_by_key(|r| std::cmp::Reverse((r.match_title, r.match_tags)));

    Ok(results)
}

pub fn refresh_notes() -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;
    
    let notes_dir = get_notes_dir()?;
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
    let full_path = get_notes_dir()?.join(path_str);
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
    let notes_root = get_notes_dir()?;
    let old_path = notes_root.join(&req.old_directory).join(&old_filename);

    let new_target_dir = notes_root.join(&req.new_directory);
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
    let notes_root = get_notes_dir()?;
    let old_path = notes_root.join(directory).join(&old_filename);

    let new_filename = new_title.replace(" ", "_") + ".md";
    let new_path = notes_root.join(directory).join(&new_filename);

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
    let notes_root = get_notes_dir()?;
    let old_path = notes_root.join(&req.old_directory).join(&filename);

    let new_target_dir = notes_root.join(&req.new_directory);
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
    let path = get_notes_dir()?.join(directory).join(filename);

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

    let path = get_notes_dir()?.join(directory);
    if path.exists() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())?;
    }

    let (mut notes, settings) = read_db();
    notes.retain(|n| n.directory != directory);
    write_db(notes, settings)
}

pub fn rename_directory(old_name: &str, new_name: &str) -> Result<(), String> {
    let _lock = MU.lock().map_err(|e| e.to_string())?;

    let notes_root = get_notes_dir()?;
    let old_path = notes_root.join(old_name);
    let new_path = notes_root.join(new_name);

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

pub fn save_settings(provider: String, model: String, notes_dir: String) -> Result<(), String> {
    let notes_dir = notes_dir.trim().to_string();
    if notes_dir.is_empty() {
        return Err("Notes folder is not configured. Please choose one in the app.".to_string());
    }
    // Reject nonexistent folders early so list/save don't silently write elsewhere.
    let path = PathBuf::from(expand_home(&notes_dir));
    if !path.is_dir() {
        return Err(format!("Notes folder does not exist: {}", path.display()));
    }
    let _lock = MU.lock().map_err(|e| e.to_string())?;
    let (notes, _) = read_db();
    write_db(notes, Settings { provider, model, notes_dir })
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

    #[test]
    fn test_extract_tags() {
        let tags = extract_tags("#work some text #City_Tour ## Sub Heading  #trailing,");
        assert!(tags.contains(&"work".to_string()));
        assert!(tags.contains(&"City_Tour".to_string()));
        assert!(tags.contains(&"trailing".to_string()));
        assert!(!tags.contains(&"Sub".to_string()));
    }

    #[test]
    fn test_search_notes() {
        let _dir = setup_test();

        save_note("#work plan\nImportant meeting details here", "Work Plan", "").expect("Save failed");
        save_note("Talk about #rustlang compiler today", "Rust Tips", "drafts").expect("Save failed");
        save_note("Completely unrelated body text", "Other Note", "").expect("Save failed");

        // Empty query returns nothing
        assert!(search_notes("").expect("Search failed").is_empty());
        assert!(search_notes("  ").expect("Search failed").is_empty());

        // Content match
        let res = search_notes("meeting").expect("Search failed");
        assert_eq!(res.len(), 1);
        assert!(res[0].match_content);
        assert!(!res[0].match_title && !res[0].match_tags);
        assert_eq!(res[0].path, "Work_Plan.md");
        assert_eq!(res[0].directory, "");
        assert!(res[0].snippet.contains("meeting"));

        // Tag match (the hashtag text is also part of the content, so
        // match_content is true as well)
        let res = search_notes("rustlang").expect("Search failed");
        assert_eq!(res.len(), 1);
        assert!(res[0].match_tags);
        assert!(res[0].match_content);
        assert_eq!(res[0].tags, vec!["rustlang"]);
        assert_eq!(res[0].directory, "drafts");

        // Case-insensitive title match
        let res = search_notes("work plan").expect("Search failed");
        assert_eq!(res.len(), 1);
        assert!(res[0].match_title);

        // Title matches rank above content matches
        save_note("agenda content about meeting", "Meeting Agenda", "").expect("Save failed");
        let res = search_notes("meeting").expect("Search failed");
        assert!(res.len() >= 2);
        assert_eq!(res[0].title, "Meeting Agenda");
        assert!(res[0].match_title);
        assert!(res[1].match_content);

        // No match at all
        assert!(search_notes("zzzznotfound").expect("Search failed").is_empty());
    }
}
