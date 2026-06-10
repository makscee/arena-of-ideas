import { defineConfig } from "vitest/config";

// vite.config.ts sets root: "web" for the web shell; vitest would inherit that
// and find no tests. This file takes priority for vitest and keeps the repo
// root, so the kernel suite under src/ runs as before.
export default defineConfig({});
