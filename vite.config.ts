import { defineConfig } from "vite";

// The web shell lives in web/; the kernel stays a plain TS library in src/.
// Build output goes to dist/ at the repo root (gitignored).
export default defineConfig({
  root: "web",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
  },
});
