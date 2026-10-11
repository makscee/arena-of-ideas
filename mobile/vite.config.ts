import { defineConfig } from "vite";
import { mcpBundle } from "../scripts/mcp-bundle";

// The phone client (mission #574). Served at /arena/ on the m1 test instance,
// so assets and API calls resolve under that base. Dev: `npm run mvp:dev`
// proxies the API to the MVP server (MVP_SERVER_URL, default :8791).
const server = process.env.MVP_SERVER_URL ?? "http://127.0.0.1:8791";

export default defineConfig({
  // arena-mcp (#959) rides along: served at /arena-mcp.mjs beside the client.
  plugins: [{ name: "arena-mcp", generateBundle() { this.emitFile({ type: "asset", fileName: "arena-mcp.mjs", source: mcpBundle() }); } }],
  root: __dirname,
  base: "/arena/",
  build: { outDir: "dist", emptyOutDir: true, target: "es2022" },
  server: { proxy: { "/arena/api": server } },
});
