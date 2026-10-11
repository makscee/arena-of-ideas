// arena-mcp as one file (#959): clients/mcp/main.ts and everything it imports
// (clients/core, the shared text helpers), bundled for Node 20+. mvp:build puts
// it in mobile/dist (mobile/vite.config.ts), so the game server serves it at
// /arena-mcp.mjs; `npm run mcp:build -- out.mjs` writes it anywhere.
import { buildSync } from "esbuild";
import { writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));

/** The bundle's source text. */
export function mcpBundle(): string {
  const r = buildSync({
    entryPoints: [`${root}clients/mcp/main.ts`],
    bundle: true,
    write: false,
    platform: "node",
    format: "esm",
    target: "node20",
    legalComments: "none",
    logLevel: "error",
  });
  return r.outputFiles[0]!.text;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const out = process.argv[2] ?? `${root}mobile/dist/arena-mcp.mjs`;
  writeFileSync(out, mcpBundle());
  console.log(`wrote ${out}`);
}
