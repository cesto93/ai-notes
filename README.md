# ai-notes

This project is a simple note-taking application that uses generative AI to help you process and organize your notes. You can write a note, and the application will either summarize it or paraphrase it for you. It also automatically extracts a title and saves it.

## Features

*   **Note Taking:** A simple web interface to write and save your notes.
*   **AI-Powered Summarization:** Automatically generate a concise summary of your notes.
*   **AI-Powered Paraphrasing:** Rephrase your notes to make them clearer and more understandable.
*   **Automatic Metadata Extraction:** The application automatically extracts a title and directory from your notes.
*   **Storage:** Notes are saved as markdown files, and metadata is stored in a TinyDB database.

## Technology Stack

*   **Backend:** Python
*   **Web Framework:** Streamlit
*   **AI/LLM Framework:** LangChain and LangGraph
*   **Generative AI Model:** Google Gemini
*   **Database:** TinyDB

## How to Run

1.  Install the dependencies:
    ```bash
    pip install -r requirements.txt
    ```
2.  Set up your Google API key in a `.env` file:
    ```
    GOOGLE_API_KEY="YOUR_API_KEY"
    ```
3.  Run the Streamlit application:
    ```bash
    streamlit run src/ui.py
    ```