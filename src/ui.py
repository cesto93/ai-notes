import streamlit as st
from langchain_google_genai import ChatGoogleGenerativeAI
import os
from src.config import load_config
from src.genai import create_agent, get_initial_state
from src.storage import save_note

load_config()

llm = ChatGoogleGenerativeAI(model="gemini-2.0-flash")

st.title("AI Notes")

if "show_input" not in st.session_state:
    st.session_state.show_input = True

if st.session_state.show_input:
    # Note input
    note = st.text_area("Write your note here:", height=150)
    agent = create_agent()
    initial_state = get_initial_state(model="gemini-2.5-flash", note=note, action="")

    if st.button("Summarize and save Note"):
        if note.strip():
            initial_state["action"] = "summarize"
            result = agent.invoke(initial_state)
            st.subheader("Summary")
            st.write(result["note"])
        else:
            st.warning("Please enter a note to summarize.")
    if st.button("Paraphrase and save Note"):
        if note.strip():
            try:
                initial_state["action"] = "paraphrase"
                result = agent.invoke(initial_state)
                st.success("Note saved successfully!")
                st.subheader("Extracted Metadata")
                st.write(f"Tags: {', '.join(result['metadata'].Tags)}")
                st.write(f"Title: {result['metadata'].Title}")
            except ValueError as e:
                st.error(str(e))
        else:
            st.warning("Please enter a note to extract metadata from.")

    if st.button("Ask and Save as Note"):
        if note.strip():
            with st.spinner("Asking the LLM..."):
                response = llm.ainvoke(note)
                save_note(response.content, note, ["llm_response"], "AI_Responses")
                st.success("Response saved as a note!")
        else:
            st.warning("Please enter a question to ask.")


def display_notes():
    """
    Displays the list of notes, making them clickable.
    When a note is clicked, its content is displayed.
    """
    st.sidebar.title("Your Notes")
    if st.sidebar.button("New Note"):
        st.session_state.show_input = True
        st.session_state.selected_note = None

    notes_dir = "notes"
    if "selected_note" not in st.session_state:
        st.session_state.selected_note = None

    if os.path.exists(notes_dir) and os.path.isdir(notes_dir):
        # List top-level files
        top_level_files = [f for f in os.listdir(notes_dir) if os.path.isfile(os.path.join(notes_dir, f)) and f.endswith(".md")]
        for note_file in top_level_files:
            note_title = os.path.splitext(note_file)[0]
            if st.sidebar.button(note_title, key=note_file):
                st.session_state.selected_note = note_file
                st.session_state.show_input = False
        
        # List subdirectories
        subdirs = [d for d in os.listdir(notes_dir) if os.path.isdir(os.path.join(notes_dir, d)) and not d.startswith(".")]
        for subdir in sorted(subdirs):
            with st.sidebar.expander(subdir):
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
                else:
                    st.write("No notes.")
    else:
        st.sidebar.write("No notes found.")

    if st.session_state.selected_note:
        note_path = os.path.join(notes_dir, st.session_state.selected_note)
        with open(note_path, "r", encoding="utf-8") as f:
            note_content = f.read()
        st.subheader(f"Note: {st.session_state.selected_note}")
        st.markdown(note_content)


display_notes()
