# 🧠 AI Notes (Tauri Edition)

AI Notes is a modern, AI-enhanced **desktop** application designed to help you capture, organize, and process your thoughts. It combines a high-performance **Rust** and **Tauri** backend with a sleek **Svelte 5** frontend, leveraging Google Gemini to provide intelligent features like summarization, paraphrasing, and mindmap generation.

## ✨ Features

- 📝 **Structured Note-Taking:** Organize your notes in a hierarchical folder structure.
- 🤖 **AI-Powered processing:**
    - **Summarization:** Condense long notes into concise summaries.
    - **Paraphrasing:** Rewrite notes for better clarity and flow.
    - **Mindmaps:** Automatically generate Mermaid.js mindmaps from your content.
- ⚡ **Native Performance:** Built with Tauri for a lightweight and fast desktop experience.
- 📂 **Local-First:** Your notes are stored as Markdown files on your own machine.
- 🗃️ **Simple Storage:** Uses Markdown for content and a local JSON file for metadata/settings.

## 🛠️ Technology Stack

- **Backend:** [Tauri](https://tauri.app/) & [Rust](https://www.rust-lang.org/)
- **Frontend:** [Svelte 5](https://svelte.dev/) (Vite / TypeScript / Tailwind CSS)
- **AI Integration:** [Google Gemini API](https://ai.google.dev/)
- **Package Manager:** [Bun](https://bun.sh/)
- **Styling:** Tailwind CSS
- **Diagrams:** [Mermaid.js](https://mermaid.js.org/)

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed
- [Bun](https://bun.sh/) installed
- (Optional) [Make](https://www.gnu.org/software/make/) for easier commands

### Local Development

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/ai-notes.git
   cd ai-notes
   ```

2. **Set up Environment Variables:**
   Create a `.env` file in the root directory (or in `ai-notes-app-tauri/src-tauri/`) with your Google API key:
   ```env
   GOOGLE_API_KEY="your-google-api-key"
   ```

3. **Run the application:**
   Using Makefile:
   ```bash
   make run
   ```
   Or manually:
   ```bash
   cd ai-notes-app-tauri
   bun install
   bun run tauri dev
   ```

### Building for Production

```bash
make build
```
This will generate the native installers for your operating system in `ai-notes-app-tauri/src-tauri/target/release/bundle/`.

## 📂 Project Structure

- `ai-notes-app-tauri/`: Main application directory.
    - `src/`: Frontend Svelte 5 application.
    - `src-tauri/`: Backend Rust application logic.
- `notes/`: Directory where your markdown notes are stored.
- `notes_db.json`: Local JSON file for metadata and settings.

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.