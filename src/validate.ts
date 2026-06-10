// Content validator — stands in front of the kernel, never inside it.
// Rules are data (SPEC §0.1), so a typo is data too: without this gate a
// misspelled effect kind, a wrong-context part, or a dangling status reference
// is silently inert — the battle runs and the ability just never fires.
// This module rejects such content with a specific, path-addressed error
// before it reaches battle(). It is the embryo of the sim-gate content linter.
//
// Pure and framework-free, like the kernel. It validates *meaning*, not just
// shape: every check here corresponds to a way the interpreter would silently
// ignore the data (see runEffect/runInterceptor default arms, evalAmount's
// stacks-on-unit zero, the registry lookups).

import { TEAM_SIZE } from "./battle.js";
import type { StatusRegistry, UnitDef } from "./types.js";

export interface ValidationIssue {
  path: string; // e.g. "teamA[0].abilities[0].effects[1]"
  message: string;
}

export class ValidationError extends Error {
  readonly issues: ValidationIssue[];
  constructor(issues: ValidationIssue[]) {
    super(`invalid content (${issues.length} issue${issues.length === 1 ? "" : "s"}):\n` +
      issues.map((i) => `  ${i.path}: ${i.message}`).join("\n"));
    this.name = "ValidationError";
    this.issues = issues;
  }
}

// ---------- the vocabulary the interpreter actually understands ----------

const EVENT_NAMES = ["BattleStart", "TurnStart", "TurnEnd", "Strike", "Hurt", "Heal", "Death", "Summon", "StatusApplied", "StatusRemoved"] as const;
const WHEN_KINDS = ["trigger", "interceptor"] as const;
const UNIT_FILTERS = ["holder", "ally", "enemy", "any"] as const;
const CONDITION_KINDS = ["holderHpAtMost"] as const;
const SELECTOR_KINDS = ["holder", "eventUnit", "frontEnemy", "allEnemies", "allAllies", "randomEnemy", "lastDeadAlly"] as const;
const AMOUNT_KINDS = ["const", "stat", "level", "stacks"] as const;
const STAT_NAMES = ["hp", "pwr"] as const;
// Which atoms run in which context (runEffect vs runInterceptor — the other side is a silent no-op).
const TRIGGER_EFFECTS = ["damage", "heal", "applyStatus", "consumeStacks", "summon", "silence", "resurrect"] as const;
const INTERCEPTOR_EFFECTS = ["cancel", "absorbHurt", "preventDeathHeal"] as const;
// Pattern fields each event admits, beyond "on" (an unknown field is a typo that silently broadens the match).
const PATTERN_FIELDS: Record<(typeof EVENT_NAMES)[number], string[]> = {
  BattleStart: [], TurnStart: [], TurnEnd: [],
  Strike: ["striker"],
  Hurt: ["unit"], Heal: ["unit"], Death: ["unit"], Summon: ["unit"],
  StatusApplied: ["unit", "status"], StatusRemoved: ["unit", "status"],
};

/** Where an ability lives: on a unit, or on a status (whose stacks "own"-references can read). */
type Owner = "unit" | "status";

// ---------- entry points ----------

/** Validate a team's units against a status registry. Returns all issues found (empty = valid). */
export function validateTeam(units: unknown, registry: StatusRegistry, label = "team"): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  if (!Array.isArray(units)) {
    issues.push({ path: label, message: "expected an array of units" });
    return issues;
  }
  if (units.length === 0 || units.length > TEAM_SIZE) {
    issues.push({ path: label, message: `a team has 1..${TEAM_SIZE} units, got ${units.length}` });
  }
  units.forEach((u, i) => validateUnit(u, registry, `${label}[${i}]`, issues));
  return issues;
}

/** Validate a draft pool: every unit content-valid (the same per-unit gate as
 * validateTeam, without its 1..TEAM_SIZE cap — a pool is a market, not a line)
 * and every name unique, because the shop stacks copies by name: two pool
 * units sharing a name would silently stack into each other on buy. */
export function validatePool(units: unknown, registry: StatusRegistry, label = "pool"): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  if (!Array.isArray(units)) {
    issues.push({ path: label, message: "expected an array of units" });
    return issues;
  }
  if (units.length === 0) {
    issues.push({ path: label, message: "a pool needs ≥1 units" });
  }
  const seen = new Map<string, number>();
  units.forEach((u, i) => {
    validateUnit(u, registry, `${label}[${i}]`, issues);
    const name = isObject(u) && typeof u["name"] === "string" ? u["name"] : undefined;
    if (name === undefined) return;
    const first = seen.get(name);
    if (first !== undefined) {
      issues.push({ path: `${label}[${i}].name`, message: `duplicate unit name "${name}" (also at ${label}[${first}]) — the shop stacks copies by name, so these would silently merge` });
    } else {
      seen.set(name, i);
    }
  });
  return issues;
}

/** Validate the status registry itself — each StatusDef is content and can be just as typo'd. */
export function validateRegistry(registry: StatusRegistry, label = "registry"): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  for (const [key, def] of Object.entries(registry)) {
    const path = `${label}.${key}`;
    if (!isObject(def)) {
      issues.push({ path, message: "StatusDef must be an object" });
      continue;
    }
    if (def.name !== key) {
      issues.push({ path, message: `StatusDef.name "${String(def.name)}" must equal its registry key "${key}" — the engine keys instances by name` });
    }
    if (def.statMods !== undefined) {
      if (!isObject(def.statMods)) {
        issues.push({ path: `${path}.statMods`, message: "statMods must be an object" });
      } else {
        for (const [stat, v] of Object.entries(def.statMods)) {
          if (!oneOf(stat, STAT_NAMES)) issues.push({ path: `${path}.statMods`, message: `unknown stat "${stat}" — stats are ${list(STAT_NAMES)}` });
          else if (typeof v !== "number") issues.push({ path: `${path}.statMods.${stat}`, message: "statMod must be a number" });
        }
      }
    }
    if (!Array.isArray(def.abilities)) {
      issues.push({ path: `${path}.abilities`, message: "StatusDef.abilities must be an array (may be empty)" });
    } else {
      def.abilities.forEach((ab, i) => validateAbility(ab, registry, "status", `${path}.abilities[${i}]`, issues));
    }
  }
  return issues;
}

/** Throw a ValidationError (listing every issue) if the team or registry is invalid. */
export function assertValidContent(units: unknown, registry: StatusRegistry, label = "team"): asserts units is UnitDef[] {
  const issues = [...validateRegistry(registry), ...validateTeam(units, registry, label)];
  if (issues.length > 0) throw new ValidationError(issues);
}

/** The assertValidContent of pools — same gate, validatePool's rules. */
export function assertValidPool(units: unknown, registry: StatusRegistry, label = "pool"): asserts units is UnitDef[] {
  const issues = [...validateRegistry(registry), ...validatePool(units, registry, label)];
  if (issues.length > 0) throw new ValidationError(issues);
}

// ---------- units ----------

function validateUnit(u: unknown, registry: StatusRegistry, path: string, issues: ValidationIssue[]): void {
  if (!isObject(u)) {
    issues.push({ path, message: "unit must be an object" });
    return;
  }
  if (typeof u["name"] !== "string" || u["name"].length === 0) {
    issues.push({ path: `${path}.name`, message: "unit name must be a non-empty string" });
  }
  const base = u["base"];
  if (!isObject(base)) {
    issues.push({ path: `${path}.base`, message: "base must be an object with hp and pwr" });
  } else {
    for (const stat of STAT_NAMES) {
      const v = base[stat];
      if (typeof v !== "number" || !Number.isInteger(v) || v < 0) {
        issues.push({ path: `${path}.base.${stat}`, message: `${stat} must be a non-negative integer, got ${JSON.stringify(v)}` });
      }
    }
  }
  if (u["level"] !== undefined && (typeof u["level"] !== "number" || !Number.isInteger(u["level"]) || u["level"] < 1)) {
    issues.push({ path: `${path}.level`, message: "level must be a positive integer" });
  }
  if (u["statuses"] !== undefined) {
    if (!Array.isArray(u["statuses"])) {
      issues.push({ path: `${path}.statuses`, message: "statuses must be an array of { status, stacks }" });
    } else {
      u["statuses"].forEach((s, i) => {
        const p = `${path}.statuses[${i}]`;
        if (!isObject(s)) {
          issues.push({ path: p, message: "status bundle must be an object { status, stacks }" });
          return;
        }
        checkStatusRef(s["status"], registry, p, issues);
        if (typeof s["stacks"] !== "number" || !Number.isInteger(s["stacks"]) || s["stacks"] < 1) {
          issues.push({ path: `${p}.stacks`, message: "stacks must be a positive integer" });
        }
      });
    }
  }
  if (u["abilities"] !== undefined) {
    if (!Array.isArray(u["abilities"])) {
      issues.push({ path: `${path}.abilities`, message: "abilities must be an array" });
    } else {
      u["abilities"].forEach((ab, i) => validateAbility(ab, registry, "unit", `${path}.abilities[${i}]`, issues));
    }
  }
}

// ---------- abilities ----------

function validateAbility(ab: unknown, registry: StatusRegistry, owner: Owner, path: string, issues: ValidationIssue[]): void {
  if (!isObject(ab)) {
    issues.push({ path, message: "ability must be an object" });
    return;
  }

  // whens — and the set of contexts this ability can fire in.
  const contexts = new Set<"trigger" | "interceptor">();
  if (!Array.isArray(ab["whens"]) || ab["whens"].length === 0) {
    issues.push({ path: `${path}.whens`, message: "an ability needs ≥1 whens" });
  } else {
    ab["whens"].forEach((w, i) => {
      const p = `${path}.whens[${i}]`;
      if (!isObject(w)) {
        issues.push({ path: p, message: "when must be an object { kind, on }" });
        return;
      }
      if (!oneOf(w["kind"], WHEN_KINDS)) {
        issues.push({ path: `${p}.kind`, message: `unknown when kind ${JSON.stringify(w["kind"])} — must be ${list(WHEN_KINDS)}` });
      } else {
        contexts.add(w["kind"]);
      }
      validatePattern(w["on"], registry, `${p}.on`, issues);
    });
  }

  // condition
  if (ab["condition"] !== undefined) {
    const p = `${path}.condition`;
    if (!isObject(ab["condition"]) || !oneOf(ab["condition"]["kind"], CONDITION_KINDS)) {
      issues.push({ path: p, message: `unknown condition kind — conditions are ${list(CONDITION_KINDS)}` });
    } else if (typeof ab["condition"]["value"] !== "number") {
      issues.push({ path: `${p}.value`, message: "condition value must be a number" });
    }
  }

  // selectors
  if (!Array.isArray(ab["selectors"]) || ab["selectors"].length === 0) {
    issues.push({ path: `${path}.selectors`, message: "an ability needs ≥1 selectors" });
  } else {
    ab["selectors"].forEach((s, i) => {
      const kind = isObject(s) ? s["kind"] : undefined;
      if (!oneOf(kind, SELECTOR_KINDS)) {
        issues.push({ path: `${path}.selectors[${i}]`, message: `unknown selector kind ${JSON.stringify(kind)} — selectors are ${list(SELECTOR_KINDS)}` });
      }
    });
  }

  // effects — kind, context fit, and per-kind references
  if (!Array.isArray(ab["effects"]) || ab["effects"].length === 0) {
    issues.push({ path: `${path}.effects`, message: "an ability needs ≥1 effects" });
  } else {
    ab["effects"].forEach((e, i) => validateEffect(e, registry, owner, contexts, `${path}.effects[${i}]`, issues));
  }
}

function validatePattern(on: unknown, registry: StatusRegistry, path: string, issues: ValidationIssue[]): void {
  if (!isObject(on) || typeof on["on"] !== "string") {
    issues.push({ path, message: 'event pattern must be an object like { on: "Hurt", unit: "holder" }' });
    return;
  }
  const name = on["on"];
  if (!oneOf(name, EVENT_NAMES)) {
    issues.push({ path: `${path}.on`, message: `unknown event ${JSON.stringify(name)} — patterns can match ${list(EVENT_NAMES)}` });
    return;
  }
  const allowed = PATTERN_FIELDS[name];
  for (const key of Object.keys(on)) {
    if (key === "on") continue;
    if (!allowed.includes(key)) {
      issues.push({ path: `${path}.${key}`, message: `"${name}" patterns take no "${key}" field${allowed.length > 0 ? ` (only ${allowed.map((f) => `"${f}"`).join(", ")})` : ""} — a typo here silently broadens the match` });
      continue;
    }
    if (key === "status") checkStatusRef(on[key], registry, path, issues);
    else if (!oneOf(on[key], UNIT_FILTERS)) {
      issues.push({ path: `${path}.${key}`, message: `unknown unit filter ${JSON.stringify(on[key])} — filters are ${list(UNIT_FILTERS)}` });
    }
  }
}

// ---------- effects ----------

function validateEffect(e: unknown, registry: StatusRegistry, owner: Owner, contexts: Set<"trigger" | "interceptor">, path: string, issues: ValidationIssue[]): void {
  if (!isObject(e) || typeof e["kind"] !== "string") {
    issues.push({ path, message: "effect must be an object with a kind" });
    return;
  }
  const kind = e["kind"];
  const isTrigger = oneOf(kind, TRIGGER_EFFECTS);
  const isInterceptor = oneOf(kind, INTERCEPTOR_EFFECTS);
  if (!isTrigger && !isInterceptor) {
    issues.push({ path: `${path}.kind`, message: `unknown effect kind "${kind}" — trigger-context: ${list(TRIGGER_EFFECTS)}; interceptor-context: ${list(INTERCEPTOR_EFFECTS)}` });
    return;
  }
  // Context fit: an atom offered only the wrong context can never run (runEffect/runInterceptor default arms).
  if (isTrigger && !contexts.has("trigger") && contexts.size > 0) {
    issues.push({ path, message: `"${kind}" is a trigger-context effect, but this ability has only interceptor whens — it can never run` });
  }
  if (isInterceptor && !contexts.has("interceptor") && contexts.size > 0) {
    issues.push({ path, message: `"${kind}" is an interceptor-context effect, but this ability has only trigger whens — it can never run` });
  }

  switch (kind) {
    case "damage":
      return validateAmount(e["amount"], owner, `${path}.amount`, issues);
    case "heal":
      return validateAmount(e["amount"], owner, `${path}.amount`, issues);
    case "applyStatus":
      checkStatusRef(e["status"], registry, path, issues);
      return validateAmount(e["stacks"], owner, `${path}.stacks`, issues);
    case "consumeStacks":
      if (e["status"] !== undefined) checkStatusRef(e["status"], registry, path, issues);
      else if (owner === "unit") {
        issues.push({ path, message: 'consumeStacks without "status" consumes the owning status — but this ability lives on a unit, so it silently does nothing' });
      }
      return validateAmount(e["stacks"], owner, `${path}.stacks`, issues);
    case "summon":
      return validateUnit(e["unit"], registry, `${path}.unit`, issues);
    case "silence":
      return;
    case "resurrect":
      return validateAmount(e["hp"], owner, `${path}.hp`, issues);
    case "cancel":
      if (e["consumeSelf"] !== undefined && owner === "unit") {
        issues.push({ path: `${path}.consumeSelf`, message: "consumeSelf consumes the owning status's stacks — but this ability lives on a unit, so it silently does nothing" });
      }
      return;
    case "absorbHurt":
      if (owner === "unit") {
        issues.push({ path, message: "absorbHurt absorbs with the owning status's stacks — on a unit ability it silently does nothing" });
      }
      return;
    case "preventDeathHeal":
      if (e["removeSelf"] === true && owner === "unit") {
        issues.push({ path: `${path}.removeSelf`, message: "removeSelf removes the owning status — but this ability lives on a unit, so it silently does nothing" });
      }
      return validateAmount(e["toHp"], owner, `${path}.toHp`, issues);
  }
}

function validateAmount(a: unknown, owner: Owner, path: string, issues: ValidationIssue[]): void {
  if (!isObject(a) || typeof a["kind"] !== "string" || !oneOf(a["kind"], AMOUNT_KINDS)) {
    issues.push({ path, message: `amount must be one of kinds ${list(AMOUNT_KINDS)}` });
    return;
  }
  switch (a["kind"]) {
    case "const":
      if (typeof a["value"] !== "number") issues.push({ path: `${path}.value`, message: "const amount needs a numeric value" });
      return;
    case "stat":
      if (!oneOf(a["stat"], STAT_NAMES)) issues.push({ path: `${path}.stat`, message: `unknown stat ${JSON.stringify(a["stat"])} — stats are ${list(STAT_NAMES)}` });
      if (a["of"] !== "holder") issues.push({ path: `${path}.of`, message: 'stat amounts read the holder: of must be "holder"' });
      return;
    case "level":
      if (a["of"] !== "holder") issues.push({ path: `${path}.of`, message: 'level amounts read the holder: of must be "holder"' });
      return;
    case "stacks":
      if (owner === "unit") {
        issues.push({ path, message: "a stacks amount reads the owning status's stacks — on a unit ability it silently evaluates to 0" });
      }
      return;
  }
}

// ---------- small helpers ----------

function checkStatusRef(name: unknown, registry: StatusRegistry, path: string, issues: ValidationIssue[]): void {
  if (typeof name !== "string") {
    issues.push({ path: `${path}.status`, message: "status reference must be a string" });
    return;
  }
  if (!(name in registry)) {
    const known = Object.keys(registry);
    issues.push({ path: `${path}.status`, message: `unknown status "${name}" — registry has ${known.length > 0 ? known.map((k) => `"${k}"`).join(", ") : "no statuses"}` });
  }
}

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function oneOf<T extends string>(v: unknown, set: readonly T[]): v is T {
  return typeof v === "string" && (set as readonly string[]).includes(v);
}

function list(set: readonly string[]): string {
  return set.join(", ");
}
