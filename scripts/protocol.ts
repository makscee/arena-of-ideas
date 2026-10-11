// The protocol files (#957): protocol/schema.json (JSON Schema of every type
// exported from src/mvp/contract.ts) and protocol/openapi.json (the route
// table in src/mvp/routes.ts), generated from the TS types, no zod.
//
//   npm run protocol           writes both
//   npm run protocol -- --check  exits 1 when either is stale
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createGenerator } from "ts-json-schema-generator";
import { MVP_API_PREFIX, MVP_API_VERSION, PLAYER_HEADER, TOKEN_HEADER } from "../src/mvp/contract.js";
import { API_ROUTES } from "../src/mvp/routes.js";

const root = fileURLToPath(new URL("..", import.meta.url));
export const SCHEMA_FILE = `${root}protocol/schema.json`;
export const OPENAPI_FILE = `${root}protocol/openapi.json`;
const CONTRACT = `${root}src/mvp/contract.ts`;

type Json = Record<string, unknown>;

/** Every type contract.ts exports, in source order. */
function contractTypes(): string[] {
  const src = readFileSync(CONTRACT, "utf8");
  return [...src.matchAll(/^export (?:interface|type) (\w+)/gm)].map((m) => m[1]!);
}

export function buildSchema(): Json {
  const gen = createGenerator({ path: CONTRACT, tsconfig: `${root}tsconfig.json`, expose: "export", topRef: false, skipTypeCheck: true, additionalProperties: true, jsDoc: "basic", sortProps: true });
  const definitions: Record<string, unknown> = {};
  for (const name of contractTypes()) {
    const { definitions: defs = {}, $schema: _s, ...self } = gen.createSchema(name) as Json & { definitions?: Json };
    Object.assign(definitions, defs, { [name]: self });
  }
  const sorted = Object.fromEntries(Object.keys(definitions).sort().map((k) => [k, definitions[k]]));
  return {
    $schema: "http://json-schema.org/draft-07/schema#",
    $id: `arena-of-ideas${MVP_API_PREFIX}/schema.json`,
    title: `Arena of Ideas API v${MVP_API_VERSION}`,
    description: "Generated from src/mvp/contract.ts by scripts/protocol.ts. /api/v1 only gains things: renaming or removing one means /api/v2.",
    definitions: sorted,
  };
}

const ref = (name: string): Json => (name.endsWith("[]") ? { type: "array", items: ref(name.slice(0, -2)) } : { $ref: `#/components/schemas/${name}` });

export function buildOpenApi(schema: Json): Json {
  const defs = schema.definitions as Json;
  const components = JSON.parse(JSON.stringify(defs).replaceAll('"#/definitions/', '"#/components/schemas/')) as Json;
  const paths: Record<string, Json> = {};
  for (const r of API_ROUTES) {
    const path = MVP_API_PREFIX + r.path.replace(/:(\w+)/g, "{$1}");
    const params = [...r.path.matchAll(/:(\w+)/g)].map((m) => ({ name: m[1], in: "path", required: true, schema: { type: "string" } }));
    const ok =
      r.response === "image/png"
        ? { description: "OK", content: { "image/png": { schema: { type: "string", format: "binary" } } } }
        : { description: "OK", content: { "application/json": { schema: r.response === "application/json" ? { type: "object" } : ref(r.response) } } };
    paths[path] = {
      ...paths[path],
      [r.method.toLowerCase()]: {
        operationId: `${r.method.toLowerCase()}${r.path.replace(/[/:.-](\w)/g, (_, c: string) => c.toUpperCase())}`,
        summary: r.note.split(/(?<=\.) /)[0],
        description: r.note + (r.only ? ` Only with ${r.only === "dev" ? "MVP_DEV=1" : r.only === "fake-mail" ? "ARENA_MAIL_FAKE=1" : "ARENA_TELEGRAM_FAKE=1"}.` : ""),
        ...(params.length ? { parameters: params } : {}),
        ...(r.body ? { requestBody: { required: true, content: { "application/json": { schema: ref(r.body) } } } } : {}),
        responses: { "200": ok, default: { description: "An error: 400 unreadable, 401 no player, 403 refused, 404 not found, 409 the rules refuse, 413 body over 16 KB, 429 too many, 501 not yet.", content: { "application/json": { schema: ref("ApiError") } } } },
        ...(r.only ? { "x-only": r.only } : {}),
      },
    };
  }
  return {
    openapi: "3.1.0",
    info: {
      title: "Arena of Ideas",
      version: String(MVP_API_VERSION),
      description:
        "The game's HTTP JSON API, generated from src/mvp/routes.ts. Battle logs follow battle-events/1 (protocol/battle-events.md). " +
        "/api/v1 and battle-events/1 only gain things (fields, routes, event and type kinds); renaming or removing one means /api/v2. A client ignores fields and event types it doesn't know.",
    },
    servers: [{ url: "https://arena.makscee.ru" }],
    components: {
      schemas: components,
      securitySchemes: {
        token: { type: "apiKey", in: "header", name: TOKEN_HEADER, description: "A PlayerSession token (invite, join, email or Telegram login)." },
        player: { type: "apiKey", in: "header", name: PLAYER_HEADER, description: "A device player id, open servers only (POST /players)." },
      },
    },
    security: [{ token: [] }, { player: [] }, {}],
    paths,
  };
}

const text = (j: Json) => JSON.stringify(j, null, 2) + "\n";

export function buildProtocol(): { schema: string; openapi: string } {
  const schema = buildSchema();
  return { schema: text(schema), openapi: text(buildOpenApi(schema)) };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const built = buildProtocol();
  const files: [string, string][] = [
    [SCHEMA_FILE, built.schema],
    [OPENAPI_FILE, built.openapi],
  ];
  if (process.argv.includes("--check")) {
    const stale = files.filter(([f, t]) => {
      try {
        return readFileSync(f, "utf8") !== t;
      } catch {
        return true;
      }
    });
    for (const [f] of stale) console.error(`stale: ${f.slice(root.length)} (npm run protocol)`);
    process.exit(stale.length ? 1 : 0);
  }
  for (const [f, t] of files) writeFileSync(f, t);
  console.log(`wrote protocol/schema.json (${Object.keys(JSON.parse(built.schema).definitions).length} types), protocol/openapi.json (${API_ROUTES.length} routes)`);
}
