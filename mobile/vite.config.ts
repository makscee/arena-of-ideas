import { defineConfig } from "vite";

// The phone client (mission #574). Served at /arena/ on the m1 test instance,
// so assets and API calls resolve under that base. Dev: `npm run mvp:dev`
// proxies the API to the MVP server (MVP_SERVER_URL, default :8791).
const server = process.env.MVP_SERVER_URL ?? "http://127.0.0.1:8791";

export default defineConfig({
  root: __dirname,
  base: "/arena/",
  build: { outDir: "dist", emptyOutDir: true, target: "es2022" },
  server: { proxy: { "/arena/api": server } },
});
