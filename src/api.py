from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
import os
from typing import List, Optional

from src.config import load_config
from src.genai import get_model, summarize_text, paraphrase_text
from src.storage import save_note, update_note, get_note_metadata, create_directory, delete_note, delete_directory, get_settings, save_settings

load_config()

def get_current_llm():
    settings = get_settings()
    return get_model(provider=settings["provider"], model=settings["model"])

app = FastAPI()

# Enable CORS for Svelte development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # For development; refine for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

NOTES_DIR = "notes"

class NoteRequest(BaseModel):
    content: str
    title: str
    directory: Optional[str] = ""

class UpdateNoteRequest(BaseModel):
    old_title: str
    old_directory: str
    new_content: str
    new_title: str
    new_directory: str

class SummarizeRequest(BaseModel):
    text: str

class DirectoryRequest(BaseModel):
    name: str

class SettingsRequest(BaseModel):
    provider: str
    model: str

@app.get("/notes")
def list_notes():
    """Returns a tree of notes and directories."""
    if not os.path.exists(NOTES_DIR):
        return {"files": [], "directories": {}}
    
    files = [f for f in os.listdir(NOTES_DIR) if os.path.isfile(os.path.join(NOTES_DIR, f)) and f.endswith(".md")]
    directories = {}
    
    subdirs = [d for d in os.listdir(NOTES_DIR) if os.path.isdir(os.path.join(NOTES_DIR, d)) and not d.startswith(".")]
    for subdir in subdirs:
        subdir_path = os.path.join(NOTES_DIR, subdir)
        notes = [f for f in os.listdir(subdir_path) if f.endswith(".md")]
        directories[subdir] = notes
        
    return {"files": files, "directories": directories}

@app.get("/note/{path:path}")
def get_note(path: str):
    full_path = os.path.join(NOTES_DIR, path)
    if not os.path.exists(full_path):
        raise HTTPException(status_code=404, detail="Note not found")
    
    with open(full_path, "r", encoding="utf-8") as f:
        content = f.read()
    
    # Extract metadata
    path_parts = path.split(os.sep)
    if len(path_parts) > 1:
        directory = path_parts[0]
        filename = path_parts[1]
    else:
        directory = ""
        filename = path_parts[0]
    
    filename_no_ext = os.path.splitext(filename)[0]
    metadata = get_note_metadata(filename_no_ext, directory)
    
    return {
        "content": content,
        "title": metadata["argument"] if metadata else filename_no_ext,
        "directory": metadata["directory"] if metadata else directory
    }

@app.post("/note")
def create_note_endpoint(req: NoteRequest):
    save_note(req.content, req.title, req.directory)
    return {"message": "Note saved"}

@app.put("/note")
def update_note_endpoint(req: UpdateNoteRequest):
    update_note(
        old_title=req.old_title,
        old_directory=req.old_directory,
        new_content=req.new_content,
        new_title=req.new_title,
        new_directory=req.new_directory
    )
    return {"message": "Note updated"}

@app.post("/directory")
def create_dir_endpoint(req: DirectoryRequest):
    create_directory(req.name)
    return {"message": "Directory created"}

@app.delete("/note/{directory}/{title}")
def delete_note_endpoint(directory: str, title: str):
    # If directory is "none", it means no directory
    actual_dir = "" if directory == "none" else directory
    delete_note(title, actual_dir)
    return {"message": "Note deleted"}

@app.delete("/note/{title}")
def delete_note_root_endpoint(title: str):
    delete_note(title, "")
    return {"message": "Note deleted"}

@app.delete("/directory/{name}")
def delete_dir_endpoint(name: str):
    delete_directory(name)
    return {"message": "Directory deleted"}

@app.get("/settings")
def get_settings_endpoint():
    return get_settings()

@app.post("/settings")
def save_settings_endpoint(req: SettingsRequest):
    save_settings(req.provider, req.model)
    return {"message": "Settings saved"}

@app.post("/summarize")
def summarize_endpoint(req: SummarizeRequest):
    llm = get_current_llm()
    result = summarize_text(llm, req.text)
    return {"result": result}

@app.post("/paraphrase")
def paraphrase_endpoint(req: SummarizeRequest):
    llm = get_current_llm()
    result = paraphrase_text(llm, req.text)
    return {"result": result}
