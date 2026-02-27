mod models;
mod storage;
mod ai;

use models::{Note, UpdateNoteRequest, MoveNoteRequest, NoteListResponse, Settings};

#[tauri::command]
fn refresh_notes() -> Result<(), String> {
    storage::refresh_notes()
}

#[tauri::command]
fn list_notes() -> Result<NoteListResponse, String> {
    storage::list_notes()
}

#[tauri::command]
fn get_note(path: String) -> Result<Note, String> {
    storage::get_note(&path)
}

#[tauri::command]
fn save_note(content: String, title: String, directory: String) -> Result<(), String> {
    storage::save_note(&content, &title, &directory)
}

#[tauri::command]
fn update_note(req: UpdateNoteRequest) -> Result<(), String> {
    storage::update_note(req)
}

#[tauri::command]
fn rename_note(old_title: String, directory: String, new_title: String) -> Result<(), String> {
    storage::rename_note(&old_title, &directory, &new_title)
}

#[tauri::command]
fn move_note(req: MoveNoteRequest) -> Result<(), String> {
    storage::move_note(req)
}

#[tauri::command]
fn delete_note(title: String, directory: String) -> Result<(), String> {
    storage::delete_note(&title, &directory)
}

#[tauri::command]
fn create_directory(name: String) -> Result<(), String> {
    storage::create_directory(&name)
}

#[tauri::command]
fn delete_directory(name: String) -> Result<(), String> {
    storage::delete_directory(&name)
}

#[tauri::command]
fn rename_directory(old_name: String, new_name: String) -> Result<(), String> {
    storage::rename_directory(&old_name, &new_name)
}

#[tauri::command]
fn get_settings() -> Result<Settings, String> {
    storage::get_settings()
}

#[tauri::command]
fn save_settings(provider: String, model: String) -> Result<(), String> {
    storage::save_settings(provider, model)
}

#[tauri::command]
async fn summarize(text: String) -> Result<String, String> {
    let settings = storage::get_settings()?;
    ai::summarize(text, settings).await
}

#[tauri::command]
async fn paraphrase(text: String) -> Result<String, String> {
    let settings = storage::get_settings()?;
    ai::paraphrase(text, settings).await
}

#[tauri::command]
async fn mindmap(text: String) -> Result<String, String> {
    let settings = storage::get_settings()?;
    ai::generate_mindmap(text, settings).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  dotenvy::dotenv().ok();
  
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().build())
    .invoke_handler(tauri::generate_handler![
        list_notes,
        get_note,
        save_note,
        update_note,
        rename_note,
        move_note,
        delete_note,
        create_directory,
        delete_directory,
        rename_directory,
        get_settings,
        save_settings,
        summarize,
        paraphrase,
        summarize,
        paraphrase,
        mindmap,
        refresh_notes
    ])
    .setup(|_app| {
      if cfg!(debug_assertions) {
        // Log plugin already added above for all handlers
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
