import streamlit as st
from langchain_google_genai import ChatGoogleGenerativeAI
import os
from src.config import load_config
from src.genai import create_agent, get_initial_state
from src.storage import save_note, update_note, get_note_metadata, create_directory

load_config()

llm = ChatGoogleGenerativeAI(model="gemini-2.0-flash")

st.title("AI Notes")

if "show_input" not in st.session_state:
    st.session_state.show_input = True

if "editing_note" not in st.session_state:
    st.session_state.editing_note = False

if st.session_state.show_input:
    # Note input
    note = st.text_area("Write your note here:", height=150)
    agent = create_agent()
    initial_state = get_initial_state(model="gemini-2.0-flash", note=note, action="")




    if st.button("Ask and Save as Note"):
        if note.strip():
            with st.spinner("Asking the LLM..."):
                response = llm.ainvoke(note)
                save_note(response.content, note, ["llm_response"], "AI_Responses")
                st.success("Response saved as a note!")
        else:
            st.warning("Please enter a question to ask.")

    st.markdown("---")
    st.subheader("Manual Note Entry")
    manual_title = st.text_input("Note Title")
    manual_directory = st.text_input("Directory (optional, e.g., 'work', 'personal')", value=st.session_state.get("prefill_directory", ""))
    manual_tags = st.text_input("Tags (comma separated, optional)")

    if st.button("Save Note Manually"):
        if note.strip() and manual_title.strip():
            tags_list = [t.strip() for t in manual_tags.split(",")] if manual_tags.strip() else []
            save_note(note, manual_title, tags_list, manual_directory.strip())
            st.success(f"Note '{manual_title}' saved successfully!")
        elif not note.strip():
            st.warning("Please enter note content above.")
        else:
            st.warning("Please provide a title for your manual note.")


def display_notes():
    """
    Displays the list of notes, making them clickable.
    When a note is clicked, its content is displayed.
    """
    st.sidebar.title("Your Notes")
    if st.sidebar.button("New Note"):
        st.session_state.show_input = True
        st.session_state.selected_note = None
        st.session_state.editing_note = False
        st.session_state.prefill_directory = ""

    notes_dir = "notes"
    if "selected_note" not in st.session_state:
        st.session_state.selected_note = None

    # Directory creation UI
    with st.sidebar.expander("📁 Create New Directory"):
        new_dir_name = st.text_input("Directory Name", key="new_dir_input")
        if st.button("Create", key="create_dir_btn"):
            if new_dir_name:
                create_directory(new_dir_name)
                st.success(f"Created '{new_dir_name}'")
                st.rerun()
            else:
                st.warning("Enter a name.")

    if os.path.exists(notes_dir) and os.path.isdir(notes_dir):
        # List top-level files
        top_level_files = [f for f in os.listdir(notes_dir) if os.path.isfile(os.path.join(notes_dir, f)) and f.endswith(".md")]
        for note_file in top_level_files:
            note_title = os.path.splitext(note_file)[0]
            if st.sidebar.button(note_title, key=note_file):
                st.session_state.selected_note = note_file
                st.session_state.show_input = False
                st.session_state.editing_note = False
        
        # List subdirectories
        subdirs = [d for d in os.listdir(notes_dir) if os.path.isdir(os.path.join(notes_dir, d)) and not d.startswith(".")]
        for subdir in sorted(subdirs):
            with st.sidebar.expander(subdir):
                if st.button(f"➕ New Note", key=f"new_note_btn_{subdir}"):
                    st.session_state.show_input = True
                    st.session_state.selected_note = None
                    st.session_state.editing_note = False
                    st.session_state.prefill_directory = subdir
                    st.rerun()
                
                subdir_path = os.path.join(notes_dir, subdir)
                note_files = [f for f in os.listdir(subdir_path) if f.endswith(".md")]
                if note_files:
                    for note_file in sorted(note_files):
                        note_title = os.path.splitext(note_file)[0]
                        # Use relative path as key and for selection
                        rel_path = os.path.join(subdir, note_file)
                        if st.button(note_title, key=rel_path):
                            st.session_state.selected_note = rel_path
                            st.session_state.show_input = False
                            st.session_state.editing_note = False
                else:
                    st.write("No notes.")
    else:
        st.sidebar.write("No notes found.")

    if st.session_state.selected_note:
        note_path = os.path.join(notes_dir, st.session_state.selected_note)
        with open(note_path, "r", encoding="utf-8") as f:
            note_content = f.read()
        
        # Parse path to get directory and filename
        path_parts = st.session_state.selected_note.split(os.sep)
        if len(path_parts) > 1:
            directory = path_parts[0]
            filename = path_parts[1]
        else:
            directory = ""
            filename = path_parts[0]
        filename_no_ext = os.path.splitext(filename)[0]
        
        metadata = get_note_metadata(filename_no_ext, directory)
        original_title = metadata["argument"] if metadata else filename_no_ext
        original_tags = metadata["tags"] if metadata else []
        original_directory = metadata["directory"] if metadata else directory

        if not st.session_state.editing_note:
            st.subheader(f"Note: {original_title}")
            col1, col2, col3 = st.columns(3)
            if col1.button("Edit Note"):
                st.session_state.editing_note = True
                st.rerun()
            
            if col2.button("Summarize"):
                agent = create_agent()
                initial_state = get_initial_state(model="gemini-2.0-flash", note=note_content, action="summarize")
                with st.spinner("Summarizing..."):
                    result = agent.invoke(initial_state)
                    st.info(result["note"])
            
            if col3.button("Paraphrase"):
                agent = create_agent()
                initial_state = get_initial_state(model="gemini-2.0-flash", note=note_content, action="paraphrase_view")
                with st.spinner("Paraphrasing..."):
                    result = agent.invoke(initial_state)
                    st.info(result["note"])
            
            st.markdown(note_content)
            if original_tags:
                st.write(f"**Tags:** {', '.join(original_tags)}")
            if original_directory:
                st.write(f"**Directory:** {original_directory}")
        else:
            st.subheader(f"Editing: {original_title}")
            new_content = st.text_area("Content", value=note_content, height=300)
            new_title = st.text_input("Title", value=original_title)
            new_directory = st.text_input("Directory", value=original_directory)
            new_tags_str = st.text_input("Tags (comma separated)", value=", ".join(original_tags))
            
            col1, col2 = st.columns(2)
            if col1.button("Save Changes"):
                new_tags = [t.strip() for t in new_tags_str.split(",")] if new_tags_str.strip() else []
                update_note(
                    old_title=original_title,
                    old_directory=original_directory,
                    new_content=new_content,
                    new_title=new_title,
                    new_tags=new_tags,
                    new_directory=new_directory
                )
                st.success("Note updated!")
                st.session_state.editing_note = False
                # Update selected_note path if title or directory changed
                new_filename = f"{new_title.replace(' ', '_')}.md"
                st.session_state.selected_note = os.path.join(new_directory, new_filename) if new_directory else new_filename
                st.rerun()
            
            if col2.button("Cancel"):
                st.session_state.editing_note = False
                st.rerun()


display_notes()
