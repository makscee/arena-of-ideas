import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { openDb } from "./db.js";
import {
  GovernanceError,
  castAoi62FixtureVotes,
  freezeSelection,
  resetAoi62Fixture,
  rollSeason,
  stageBounce,
  stageShip,
} from "./governance.js";
import { readSeasonPointer } from "./season-store.js";

function usage(): never {
  throw new GovernanceError(`usage:
  npm run govern -- status
  npm run govern -- freeze --season N --version N
  npm run govern -- ship --idea ID --candidate PATH --season N --version N
  npm run govern -- bounce --idea ID --reason TEXT --season N --version N
  npm run govern -- roll --season N --version N
  AOI_E2E=1 npm run govern -- fixture-reset --name aoi62-governance
  AOI_E2E=1 npm run govern -- fixture-votes --ship ID --bounce ID`);
}

const argv = process.argv.slice(2);
const command = argv.shift() ?? usage();
function option(name: string, required = true): string | undefined {
  const i = argv.indexOf(`--${name}`);
  const value = i >= 0 ? argv[i + 1] : undefined;
  if (required && (!value || value.startsWith("--"))) throw new GovernanceError(`missing --${name}`);
  return value;
}
function intOption(name: string): number {
  const raw = option(name)!;
  const value = Number(raw);
  if (!Number.isInteger(value) || value < 1) throw new GovernanceError(`--${name} must be a positive integer`);
  return value;
}
function expected() { return { season: intOption("season"), contentVersion: intOption("version") }; }

const path = process.env.DB_PATH ?? "./data/arena.db";
const { db, sqlite } = openDb(path);
const deps = { db, sqlite, clock: () => Math.floor(Date.now() / 1000) };

try {
  let result: unknown;
  switch (command) {
    case "status": result = readSeasonPointer(db); break;
    case "freeze": result = freezeSelection(deps, expected()); break;
    case "ship": {
      const candidatePath = resolve(option("candidate")!);
      let candidate: unknown;
      try { candidate = JSON.parse(readFileSync(candidatePath, "utf8")); }
      catch (err) { throw new GovernanceError(`cannot read candidate ${candidatePath}: ${(err as Error).message}`); }
      result = stageShip(deps, expected(), option("idea")!, candidate, candidatePath);
      break;
    }
    case "bounce": result = stageBounce(deps, expected(), option("idea")!, option("reason")!); break;
    case "roll": result = rollSeason(deps, expected()); break;
    case "fixture-reset": {
      if (option("name") !== "aoi62-governance") throw new GovernanceError("fixture-reset requires --name aoi62-governance");
      result = resetAoi62Fixture(deps, process.env.AOI_E2E === "1");
      break;
    }
    case "fixture-votes": result = castAoi62FixtureVotes(deps, process.env.AOI_E2E === "1", option("ship")!, option("bounce")!); break;
    default: usage();
  }
  console.log(JSON.stringify(result, null, 2));
} catch (err) {
  console.error(err instanceof Error ? err.message : String(err));
  process.exitCode = 1;
} finally {
  sqlite.close();
}
