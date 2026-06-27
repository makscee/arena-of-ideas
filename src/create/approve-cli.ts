/**
 * approve (PRD #013 slice 4) — `npm run approve`.
 *
 * Lists the pending candidates in candidates/ (numbered), and moves one into the
 * playable approved-units registry (registry/approved-units.json) so a new
 * browser run can draft it. The vote gate is out of scope — this is the manual
 * `approve` a human runs once they like a candidate.
 *
 * Usage (from the repo root):
 *   npm run approve                 # list pending candidates + their numbers
 *   npm run approve -- <id>         # approve candidate <id> into the registry
 *
 * Approval is bookkeeping (the gauntlet already judged): it appends the
 * candidate's units — creator credit stamped — to the registry, re-validating
 * the whole pool, and refuses a name collision with a shipped or already-approved
 * unit. A rejected/pending candidate never leaks into the playable pool because
 * only an explicitly named, parsed-and-validated candidate is ever approved.
 *
 * Exit codes: 0 = listed or approved; 1 = usage / not-found / collision.
 */

import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { stressRegistry, stressAbilities } from "../content/stress.js";
import { DEFAULT_RUN_POOL } from "../tunables.js";
import { parseApprovedRegistry } from "../registry.js";
import type { ApprovedRegistry } from "../registry.js";
import { parseCandidateRecord } from "./candidates.js";
import { approveInto } from "./approve.js";
import type { CandidateRecord } from "./provenance.js";

const CANDIDATES_DIR = "candidates";
const REGISTRY_FILE = join("registry", "approved-units.json");

/** Every candidate record under candidates/, parsed + validated, sorted by id —
 * the pending pool. A malformed file is reported and skipped, never silently
 * dropping a good one with it. Exported for the unit test. */
export function loadCandidates(dir: string): { records: CandidateRecord[]; errors: string[] } {
  const records: CandidateRecord[] = [];
  const errors: string[] = [];
  if (!existsSync(dir)) return { records, errors };
  const files = readdirSync(dir).filter((f) => f.endsWith(".json")).sort();
  for (const f of files) {
    const path = join(dir, f);
    try {
      records.push(parseCandidateRecord(JSON.parse(readFileSync(path, "utf8")), stressRegistry, stressAbilities, path));
    } catch (err) {
      errors.push(`${path}: ${(err as Error).message}`);
    }
  }
  return { records, errors };
}

/** Read the approved-units registry file, or an empty registry if absent. */
function loadRegistry(path: string): ApprovedRegistry {
  if (!existsSync(path)) return { units: [] };
  return parseApprovedRegistry(JSON.parse(readFileSync(path, "utf8")), stressRegistry, stressAbilities, path);
}

function listPending(dir: string): number {
  const { records, errors } = loadCandidates(dir);
  for (const e of errors) process.stderr.write(`[approve] skipping ${e}\n`);
  if (records.length === 0) {
    process.stdout.write("No pending candidates in candidates/.\n");
    return 0;
  }
  process.stdout.write(`Pending candidates (${records.length}):\n`);
  records.forEach((r, i) => {
    const names = r.units.map((u) => u.name).join(", ");
    process.stdout.write(
      `  ${i + 1}. ${r.id} — ${names} ` +
        `(by ${r.provenance.creator}, ${r.provenance.harness}/${r.provenance.model}, ` +
        `winRate ${r.gate.overallWinRate.toFixed(3)})\n`,
    );
  });
  process.stdout.write(`\nApprove one: npm run approve -- <id>\n`);
  return 0;
}

function main(): void {
  const argv = process.argv.slice(2);
  const repoRoot = process.cwd();
  const candidatesDir = resolve(repoRoot, CANDIDATES_DIR);
  const registryPath = resolve(repoRoot, REGISTRY_FILE);

  if (argv.length === 0) {
    process.exit(listPending(candidatesDir));
  }
  if (argv.length !== 1 || argv[0]!.startsWith("--")) {
    process.stderr.write("Usage: approve [<id>]   (no args lists pending candidates)\n");
    process.exit(1);
  }
  const id = argv[0]!;

  const { records } = loadCandidates(candidatesDir);
  const record = records.find((r) => r.id === id);
  if (record === undefined) {
    process.stderr.write(`[approve] no candidate "${id}" in candidates/ — run \`npm run approve\` to list pending.\n`);
    process.exit(1);
  }

  const current = loadRegistry(registryPath);
  const shippedNames = DEFAULT_RUN_POOL.map((u) => u.name);
  let next: ApprovedRegistry;
  try {
    next = approveInto(current, record, shippedNames, stressRegistry, stressAbilities);
  } catch (err) {
    process.stderr.write(`[approve] cannot approve "${id}": ${(err as Error).message}\n`);
    process.exit(1);
  }

  // Write the registry, preserving the leading comment so the file stays legible.
  mkdirSync(resolve(repoRoot, "registry"), { recursive: true });
  const out =
    JSON.stringify(
      {
        _comment:
          "The playable approved-units registry (PRD #013 slice 4). Passing candidates from the creation loop are approved into this file by `npm run approve`; the web run screen merges these onto the shipped DEFAULT_RUN_POOL so a new run can draft them. Each unit is a team-file UnitDef, validated like all content; an optional `_creator` field carries authorship credit the codex displays.",
        units: next.units,
      },
      null,
      2,
    ) + "\n";
  writeFileSync(registryPath, out, "utf8");

  // Report the units the approval actually added (the new names), drawn from the
  // diff between the new registry and the prior one.
  const priorNames = new Set(current.units.map((u) => u.name));
  const added = next.units.filter((u) => !priorNames.has(u.name)).map((u) => u.name).join(", ");
  process.stdout.write(
    `Approved "${id}" — ${added} (by ${record.provenance.creator}) is now draftable in a new run.\n`,
  );
  process.exit(0);
}

const isMain =
  typeof process !== "undefined" &&
  typeof import.meta?.url === "string" &&
  process.argv[1] !== undefined &&
  import.meta.url.endsWith(process.argv[1].replace(/^.*[\\/]/, ""));

if (isMain) main();
