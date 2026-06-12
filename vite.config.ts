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
  },
  server: {
    proxy: {
      "/v1": serverUrl,
      "/_mock": serverUrl,
    },
  },
});
