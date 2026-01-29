import os
import shutil
from src.storage import save_note, move_note, delete_note, delete_directory

NOTES_DIR = "notes"
DB_FILE = "notes_db.json"

def test_move_note():
    # Setup
    if os.path.exists(NOTES_DIR):
        shutil.rmtree(NOTES_DIR)
    if os.path.exists(DB_FILE):
        os.remove(DB_FILE)
    
    os.makedirs(NOTES_DIR, exist_ok=True)
    
    # Create a note in root
    save_note("Note in root content", "Root Note", "")
    
    # Create a directory
    os.makedirs(os.path.join(NOTES_DIR, "test_dir"), exist_ok=True)
    
    print("Files before move:", os.listdir(NOTES_DIR))
    
    # Move note to test_dir
    move_note("Root Note", "", "test_dir")
    
    print("Files after move:", os.listdir(NOTES_DIR))
    if os.path.exists(os.path.join(NOTES_DIR, "test_dir", "Root_Note.md")):
        print("Success: Note moved to test_dir")
    else:
        print("Failure: Note not found in test_dir")
        
    # Move it back to root
    move_note("Root Note", "test_dir", "")
    if os.path.exists(os.path.join(NOTES_DIR, "Root_Note.md")):
        print("Success: Note moved back to root")
    else:
        print("Failure: Note not found in root")

if __name__ == "__main__":
    test_move_note()
