import { defineConfig } from "vite";

// The web shell lives in web/; the kernel stays a plain TS library in src/.
// Build output goes to dist/ at the repo root (gitignored).
//
// The arena server API (#016) is reached through same-origin relative URLs
// ("/v1/..."), proxied here in dev so the client needs no CORS and no baked-in
// server origin; a deployment fronts both behind one origin the same way.
// AOI_SERVER_URL points the proxy elsewhere (the e2e harness's child server);
// /_mock is the e2e-only mailer peek (server MOCK_MODE).
const serverUrl = process.env.AOI_SERVER_URL ?? "http://localhost:8787";

export default defineConfig({
  root: "web",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    // The login boot path (web/main.ts, #016 slice 3) uses top-level await,
    // which vite's default es2020 target refuses at build time (dev never
    // hit it — esbuild only transpiles down for production builds). es2022
    // is the first target with TLA; every browser this game supports has it.
    target: "es2022",
  },
  server: {
    proxy: {
      "/v1": serverUrl,
      "/_mock": serverUrl,
    },
  },
});
