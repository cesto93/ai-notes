import streamlit as st
from langchain_google_genai import ChatGoogleGenerativeAI
from langchain_core.messages import HumanMessage
from src.config import load_config

load_config()

llm = ChatGoogleGenerativeAI(model="gemini-2.0-flash")

st.title("AI Notes")

# Note input
note = st.text_area("Write your note here:", height=150)

if st.button("Summarize Note"):
    if note.strip():
        result = llm.invoke(f"Summarize this note: {note}")
        st.subheader("Summary")
        st.write(result.content)
    else:
        st.warning("Please enter a note to summarize.")
