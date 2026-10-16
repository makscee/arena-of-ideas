import { defineConfig } from "vite";

// The phone client (mission #574). Served at /arena/ on the m1 test instance,
// so assets and API calls resolve under that base. Dev: `npm run mvp:dev`
// proxies the API to the MVP server (MVP_SERVER_URL, default :8791).
//
// `--mode cg` (npm run mvp:build:cg, mission #1214) is the CrazyGames zip: a
// relative base so it runs from any folder or host, VITE_CG=1 for the client's
// cg flag (./cg.ts), the API at VITE_API_ORIGIN (from the environment), output
// to dist-cg.
const server = process.env.MVP_SERVER_URL ?? "http://127.0.0.1:8791";

export default defineConfig(({ mode }) => {
  const cg = mode === "cg";
  return {
    root: __dirname,
    base: cg ? "./" : "/arena/",
    build: { outDir: cg ? "dist-cg" : "dist", emptyOutDir: true, target: "es2022" },
    ...(cg ? { define: { "import.meta.env.VITE_CG": JSON.stringify("1") } } : {}),
    server: { proxy: { "/arena/api": server } },
  };
});
