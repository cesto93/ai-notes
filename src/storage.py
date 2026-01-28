from typing import List
import os
from tinydb import TinyDB, Query

DB_FILE = "notes_db.json"
NOTES_DIR = "notes"


def save_note(note: str, title: str, tags: List[str], directory: str = "") -> None:
    """
    Saves a note with the given content, argument, and tags.

    Args:
        note (str): The content of the note.
        title (str): The note title.
        tags (List[str]): A list of tags associated with the note.
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
    db.insert({"argument": title, "tags": tags, "directory": directory})
def update_note(old_title: str, old_directory: str, new_content: str, new_title: str, new_tags: List[str], new_directory: str) -> None:
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
        {"argument": new_title, "tags": new_tags, "directory": new_directory},
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
