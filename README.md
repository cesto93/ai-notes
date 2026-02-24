# 🧠 AI Notes

AI Notes is a modern, AI-enhanced note-taking application designed to help you capture, organize, and process your thoughts. It combines a powerful **FastAPI** backend with a sleek **SvelteKit** frontend, leveraging Large Language Models (LLMs) to provide intelligent features like summarization, paraphrasing, and mindmap generation.

## ✨ Features

- 📝 **Structured Note-Taking:** Organize your notes in a hierarchical folder structure.
- 🤖 **AI-Powered processing:**
    - **Summarization:** Condense long notes into concise summaries.
    - **Paraphrasing:** Rewrite notes for better clarity and flow.
    - **Mindmaps:** Automatically generate Mermaid.js mindmaps from your content.
- 🔌 **Provider Agnostic:** Supports multiple LLM providers, including **Google Gemini**, **Ollama**, and **Groq**.
- 🗃️ **Lightweight Storage:** Uses Markdown files for notes and TinyDB for metadata/settings.
- 🐳 **Docker Ready:** Easily deployable via Docker and Docker Compose.

## 🛠️ Technology Stack

- **Backend:** [FastAPI](https://fastapi.tiangolo.com/) (Python)
- **Frontend:** [SvelteKit](https://kit.svelte.dev/) (TypeScript/Tailwind CSS)
- **AI Orchestration:** [LangChain](https://python.langchain.com/)
- **Package Managers:** [uv](https://github.com/astral-sh/uv) (Python) and [Bun](https://bun.sh/) (Frontend)
- **Database:** [TinyDB](https://tinydb.readthedocs.io/)

## 🚀 Getting Started

### Prerequisites

- [uv](https://github.com/astral-sh/uv) installed
- [Bun](https://bun.sh/) installed
- (Optional) Docker and Docker Compose

### Local Development

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/ai-notes.git
   cd ai-notes
   ```

2. **Set up Environment Variables:**
   Create a `.env` file in the root directory:
   ```env
   GOOGLE_API_KEY="your-google-api-key"
   GROQ_API_KEY="your-groq-api-key"
   # Add other keys as needed
   ```

3. **Install dependencies and run (using Makefile):**
   ```bash
   make run
   ```
   This will start both the FastAPI backend (port 8000) and the SvelteKit frontend (port 5173).

### Docker Deployment

To run the entire stack using Docker:
```bash
make start
```
The application will be accessible at `http://localhost:8000`.

## 📂 Project Structure

- `src/`: Backend FastAPI application logic.
- `ai-notes-app/`: Frontend SvelteKit application.
- `notes/`: Directory where your markdown notes are stored.
- `notes_db.json`: TinyDB file for metadata.

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.