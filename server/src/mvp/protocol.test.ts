// #957: the protocol files match the TS types and the route table, and the
// route table lists exactly the routes the API serves.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { MVP_API_PREFIX } from "../../../src/mvp/contract.js";
import { API_ROUTES } from "../../../src/mvp/routes.js";
import { buildProtocol, OPENAPI_FILE, SCHEMA_FILE } from "../../../scripts/protocol.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";

const app = createMvpApp({ content: mvpContent(), dev: true });

describe("protocol", () => {
  it("protocol/schema.json and openapi.json are fresh (else: npm run protocol)", () => {
    const built = buildProtocol();
    expect(readFileSync(SCHEMA_FILE, "utf8")).toBe(built.schema);
    expect(readFileSync(OPENAPI_FILE, "utf8")).toBe(built.openapi);
  }, 60_000);

  it("the route table lists exactly the routes app.ts serves", () => {
    const served = new Set(app.routes.filter((r) => r.method !== "ALL").map((r) => `${r.method} ${r.path}`));
    const listed = new Set(API_ROUTES.map((r) => `${r.method} ${MVP_API_PREFIX}${r.path}`));
    expect([...served].filter((r) => !listed.has(r))).toEqual([]);
    expect([...listed].filter((r) => !served.has(r))).toEqual([]);
  });

  it("every type a route names is in the schema, and every $ref resolves", () => {
    const schema = JSON.parse(readFileSync(SCHEMA_FILE, "utf8")) as { definitions: Record<string, unknown> };
    const named = API_ROUTES.flatMap((r) => [r.body, r.response]).filter((t): t is string => !!t && !t.includes("/")).map((t) => t.replace(/\[\]$/, ""));
    expect(named.filter((t) => !(t in schema.definitions))).toEqual([]);
    const openapi = readFileSync(OPENAPI_FILE, "utf8");
    const schemas = (JSON.parse(openapi) as { components: { schemas: Record<string, unknown> } }).components.schemas;
    const refs = [...openapi.matchAll(/"\$ref": "#\/components\/schemas\/([^"]+)"/g)].map((m) => decodeURIComponent(m[1]!));
    expect(refs.filter((r) => !(r in schemas))).toEqual([]);
    expect(JSON.stringify(schema)).not.toContain('"#/definitions/__');
  });

  it("serves both at /api/v1", async () => {
    for (const file of ["schema.json", "openapi.json"]) {
      const res = await app.request(`${MVP_API_PREFIX}/${file}`);
      expect(res.status).toBe(200);
      expect(res.headers.get("content-type")).toContain("application/json");
      expect(await res.text()).toBe(readFileSync(file === "schema.json" ? SCHEMA_FILE : OPENAPI_FILE, "utf8"));
    }
  });
});
