# AGENTS.md

Tauri 2 desktop app: Svelte 5 frontend (`src/`) + Rust backend (`src-tauri/src/`). Package manager is **Bun** (`bun.lock`) — do not use npm/pnpm.

## Commands (`Makefile` is canonical)

- `make run` / `make build` — wraps `bun run tauri dev|build`
- `make test` — `cd src-tauri && cargo test -- --test-threads=1`. Keep `--test-threads=1`: storage tests share `TEST_DIR` static + `MU` mutex.
- `make bun-check` — `svelte-check` (frontend typecheck)
- `make cargo-check` — `cargo check` (fast backend check)
- No frontend unit tests, no lint/format config.

## Architecture

- Frontend entry: `src/routes/+page.svelte`, `+layout.svelte`; SPA mode (`src/routes/+layout.ts` sets `ssr = false`, required for Tauri).
- SvelteKit 3: no `svelte.config.js` — SvelteKit/adapter-static configured inside `vite.config.js` (`fallback: "index.html"`, `frontendDist: ../build`).
- `src-tauri/src/lib.rs` (`run()` + `invoke_handler!`); `main.rs` is trivial. Logic split: `storage.rs` (notes/dirs/settings) + `ai.rs` (LLM calls) + `models.rs` (shared structs).
- Frontend↔backend bridge: `src/lib/api.ts` (`invoke(...)`) mirrors `#[tauri::command]` names in `lib.rs`. Update both together. `#lib` import alias maps to `./src/lib/*` (`package.json` `imports`).
- Diagrams are rendered by [pele](https://pele.run) (`src/lib/components/Mermaid.svelte`, kept name only as a legacy alias). Pele is synchronous — use `mount()` with an `onError` callback, not async `mermaid.render()`; theming is via `--pele-*` CSS variables, not `mermaid.initialize()`.
- Dev server: fixed port `1420` (`strictPort: true`) + HMR `1421`; Vite ignores `src-tauri/`. `lucide-svelte` is bundled via `ssr.noExternal` (extensionless-ESM workaround) — don't remove.

## Frontend gotchas

- After upgrading `@sveltejs/kit` (or other Svelte deps), delete stale generated output before dev: `rm -rf .svelte-kit build && bunx svelte-kit sync`. Old `generated/client/` trees reference removed runtime paths (e.g. `svelte-5/error.svelte`) and break `vite dev` with `Failed to resolve import` + `__sveltekit/environment` 404s. Always restart the dev server after re-installing deps.

## Backend gotchas

- Storage paths are CWD-relative: `PathBuf::from("notes")` / `"notes_db.json"` resolve to `src-tauri/notes/`, `src-tauri/notes_db.json` under `tauri dev`. Root `.gitignore` entries for `notes`/`notes_db.json` are stale paths — real files live under `src-tauri/`.
- Filename convention: `title.replace(" ", "_") + ".md"`; title recovery does reverse. `NoteMetadata.argument` is the title (legacy TinyDB name) — don't rename.
- `notes_db.json` is TinyDB-shaped (`_default` + `settings` tables); `read_db` falls back to `provider: "google", model: "gemini-2.0-flash"` when missing.
- AI provider dispatch: `ai.rs::call_llm` matches on `settings.provider` (`summarize`/`paraphrase`/`mindmap` all route through it). `google` → Gemini API; `openrouter`, `ollama`, `llamacpp` → OpenAI-compatible chat completions (`openrouter` needs `OPENROUTER_API_KEY`; `ollama` hits `localhost:11434`; `llamacpp` hits `llama-server` on `localhost:8080` — both no key). Keys loaded via `dotenvy::dotenv()` in `run()`; `.env` lives in `src-tauri/.env`. Unit tests cover only request-build/parse helpers, not the network call.
- README paths mentioning `ai-notes-app-tauri/` are stale — app root is the repo root.
