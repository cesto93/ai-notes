import streamlit as st
from langchain_google_genai import ChatGoogleGenerativeAI
from src.config import load_config
from src.genai import create_agent, summarize_text, extract_metadata, get_initial_state

load_config()

llm = ChatGoogleGenerativeAI(model="gemini-2.0-flash")

st.title("AI Notes")

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
