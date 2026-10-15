import { defineConfig } from "vite";
import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
// SvelteKit 3: project configuration lives in the vite plugin (svelte.config.js removed)
export default defineConfig(async () => ({
  plugins: [
    sveltekit({
      adapter: adapter({
        fallback: "index.html",
      }),
    }),
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // Workaround for lucide-svelte 1.0.x shipping extensionless ESM imports
  // which breaks Node strict ESM resolution in Vite SSR / SvelteKit dev.
  // Bundling it via esbuild avoids ERR_MODULE_NOT_FOUND.
  // See: https://github.com/lucide-icons/lucide/issues/4280
  ssr: {
    noExternal: ["lucide-svelte"],
  },
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
