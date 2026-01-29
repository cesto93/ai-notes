import os
from tinydb import TinyDB, Query

DB_FILE = "notes_db.json"
NOTES_DIR = "notes"


def create_directory(directory: str) -> None:
    """
    Creates a new directory in the notes folder.
    """
    notes_base_dir = os.path.join(os.path.dirname(__file__), "..", NOTES_DIR)
    target_dir = os.path.join(notes_base_dir, directory)
    os.makedirs(target_dir, exist_ok=True)


def save_note(note: str, title: str, directory: str = "") -> None:
    """
    Saves a note with the given content, argument, and directory.

    Args:
        note (str): The content of the note.
        title (str): The note title.
        directory (str): The directory where the note should be saved.
    """
    # Ensure notes directory exists
    notes_base_dir = os.path.join(os.path.dirname(__file__), "..", NOTES_DIR)
    target_dir = os.path.join(notes_base_dir, directory) if directory else notes_base_dir
    os.makedirs(target_dir, exist_ok=True)

    # Save note as markdown file
    filename = f"{title.replace(' ', '_')}.md"
    filepath = os.path.join(target_dir, filename)
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(note)

    # Save metadata to TinyDB
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    db.insert({"argument": title, "directory": directory})
def update_note(old_title: str, old_directory: str, new_content: str, new_title: str, new_directory: str) -> None:
    """
    Updates an existing note's content and metadata.
    """
    notes_base_dir = os.path.join(os.path.dirname(__file__), "..", NOTES_DIR)
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    Note = Query()

    # Find the old note in DB
    # The DB uses 'argument' for title
    db.update(
        {"argument": new_title, "directory": new_directory},
        (Note.argument == old_title) & (Note.directory == old_directory)
    )

    # Handle file system changes
    old_filename = f"{old_title.replace(' ', '_')}.md"
    old_filepath = os.path.join(notes_base_dir, old_directory, old_filename)
    
    new_target_dir = os.path.join(notes_base_dir, new_directory) if new_directory else notes_base_dir
    os.makedirs(new_target_dir, exist_ok=True)
    new_filename = f"{new_title.replace(' ', '_')}.md"
    new_filepath = os.path.join(new_target_dir, new_filename)

    # If title or directory changed, we might need to remove the old file
    if old_filepath != new_filepath and os.path.exists(old_filepath):
        os.remove(old_filepath)

    with open(new_filepath, "w", encoding="utf-8") as f:
        f.write(new_content)


def move_note(old_title: str, old_directory: str, new_directory: str) -> None:
    """
    Moves a note to a new directory.
    """
    notes_base_dir = os.path.join(os.path.dirname(__file__), "..", NOTES_DIR)
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    Note = Query()

    # Find the note metadata
    filename_no_ext = old_title.replace(' ', '_')
    
    # Update DB
    # We need to find the correct entry. The argument is the original title.
    # get_note_metadata helps us find it.
    metadata = get_note_metadata(filename_no_ext, old_directory)
    if not metadata:
        # Fallback if metadata not found
        actual_title = old_title
    else:
        actual_title = metadata['argument']

    db.update(
        {"directory": new_directory},
        (Note.argument == actual_title) & (Note.directory == old_directory)
    )

    # Handle file system changes
    old_filename = f"{filename_no_ext}.md"
    old_filepath = os.path.join(notes_base_dir, old_directory, old_filename)
    
    new_target_dir = os.path.join(notes_base_dir, new_directory) if new_directory else notes_base_dir
    os.makedirs(new_target_dir, exist_ok=True)
    new_filepath = os.path.join(new_target_dir, old_filename)

    if os.path.exists(old_filepath):
        import shutil
        shutil.move(old_filepath, new_filepath)


def get_note_metadata(filename_no_ext: str, directory: str):
    """
    Retrieves metadata for a note from TinyDB based on its filename (without extension).
    """
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    Note = Query()
    # Search for an entry where the argument (title) when processed matches filename_no_ext
    results = db.search(Note.directory == directory)
    for res in results:
        if res['argument'].replace(' ', '_') == filename_no_ext:
            return res
    return None


def delete_note(title: str, directory: str) -> None:
    """
    Deletes a note file and its metadata.
    """
    notes_base_dir = os.path.join(os.path.dirname(__file__), "..", NOTES_DIR)
    # The title passed might have underscores or spaces
    filename_no_ext = title.replace(' ', '_')
    filename = f"{filename_no_ext}.md"
    filepath = os.path.join(notes_base_dir, directory, filename)

    if os.path.exists(filepath):
        os.remove(filepath)

    # Remove metadata
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    Note = Query()
    
    # Try direct match first
    if db.contains((Note.argument == title) & (Note.directory == directory)):
        db.remove((Note.argument == title) & (Note.directory == directory))
    else:
        # Try matching by processed title (allowing for underscore/space mismatch)
        all_notes = db.search(Note.directory == directory)
        for n in all_notes:
            if n['argument'].replace(' ', '_') == filename_no_ext:
                db.remove(doc_ids=[n.doc_id])
                break


def delete_directory(directory: str) -> None:
    """
    Deletes a directory and all its contents (notes and metadata).
    """
    notes_base_dir = os.path.join(os.path.dirname(__file__), "..", NOTES_DIR)
    target_dir = os.path.join(notes_base_dir, directory)

    if os.path.exists(target_dir) and os.path.isdir(target_dir):
        import shutil
        shutil.rmtree(target_dir)

    # Remove metadata for all notes in this directory
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    Note = Query()
    db.remove(Note.directory == directory)


def get_settings():
    """Retrieves settings from TinyDB."""
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    Settings = Query()
    settings = db.table('settings').all()
    if settings:
        return settings[0]
    return {"provider": "google", "model": "gemini-2.0-flash"}


def save_settings(provider: str, model: str):
    """Saves settings to TinyDB."""
    db_path = os.path.join(os.path.dirname(__file__), "..", DB_FILE)
    db = TinyDB(db_path)
    table = db.table('settings')
    table.truncate()
    table.insert({"provider": provider, "model": model})
