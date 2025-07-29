import streamlit as st
from langchain_google_genai import ChatGoogleGenerativeAI
from src.config import load_config
from src.genai import create_agent, summarize_text, extract_metadata

load_config()

llm = ChatGoogleGenerativeAI(model="gemini-2.0-flash")

st.title("AI Notes")

# Note input
note = st.text_area("Write your note here:", height=150)
llm = create_agent(model="gemini-2.5-flash")

if st.button("Summarize Note"):
    if note.strip():
        result = summarize_text(llm, note)
        st.subheader("Summary")
        st.write(result)
    else:
        st.warning("Please enter a note to summarize.")
if st.button("Extract Metadata"):
    if note.strip():
        try:
            metadata = extract_metadata(llm, note)
            st.subheader("Extracted Metadata")
            st.write(f"Argument: {metadata.Argument}")
            st.write(f"Tags: {', '.join(metadata.Tags)}")
        except ValueError as e:
            st.error(str(e))
    else:
        st.warning("Please enter a note to extract metadata from.")
