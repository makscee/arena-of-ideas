// The Codex (rules v2, mission makscee/void-board#1043, slice #1049; design
// page §6 and §8). Pure: the pair key, a pair's state, what the forge shows
// a viewer, and the designed fusion itself. The server keeps the pairs
// (MvpStore.codexPair) and decides which design a fuse uses
// (server/src/mvp/codex.ts); the night batch (#1050) writes the candidates.
//
// - Order is dropped: A+B and B+A are one pair, keyed by the sorted ids.
// - A pair is unforged (no candidates yet), ready to claim (three sealed
//   candidates, nobody picked) or discovered (a human picked one: it is
//   canonical, with their name, final forever). Claiming gives credit only.
// - A fusion needs two Awoken (fuseCheck). The fused unit keeps the parents'
//   stats by fusedStats (the stronger plus half the other) and gets the
//   design's rule, read as one sentence.
import { fuseCheck, fusedStats } from "./forms.js";
import type { LineUnit, PlayerRef, UnitForm, UnitId } from "./contract.js";
import type { AbilityRegistry, StatusRegistry } from "../types.js";

export type PairState = "discovered" | "ready" | "unforged";

/** One designed fusion: a third creature, not a splice. The rule is the v2
 * schema only (ability names the grammar resolves); stats are never in it. */
export interface FusionDesign {
  name: string;
  emoji?: string;
  /** Which way it leans: toward the pair's `a`, toward `b`, or a twist. */
  lean: "a" | "b" | "twist";
  recipe: UnitForm;
  /** A line of flavour. */
  flavour?: string;
  texts?: { ru?: { name: string; flavour?: string } };
  /** Forged words the design uses (design §7: a claim can bring a word in). */
  words?: string[];
  /** The registry entries the rule coins (the night batch's own abilities and
   * statuses, #1050); the fused unit carries them into its fights. */
  abilities?: AbilityRegistry;
  statuses?: StatusRegistry;
}

export interface CodexClaim {
  /** Index into the pair's candidates; that one is canonical. */
  pick: number;
  by: PlayerRef;
  /** ISO time of the claim. */
  at: string;
}

/** One unordered pair. `a` < `b` (codexPair builds it). */
export interface CodexPair {
  a: UnitId;
  b: UnitId;
  /** The night batch's three candidates; empty while unforged. */
  candidates: FusionDesign[];
  /** ISO time the candidates were stored. */
  preparedAt?: string;
  /** Set once, by the first human to forge the pair while ready. */
  claim?: CodexClaim;
}

/** What the forge shows for a pair, to one viewer. Sealed candidates never
 * leave the server until the claim; only their count does. */
export interface ForgeView {
  a: UnitId;
  b: UnitId;
  state: PairState;
  /** discovered: the canonical design and its credit. */
  design?: FusionDesign;
  claimedBy?: PlayerRef;
  claimedAt?: string;
  /** ready: how many sealed designs wait (three). */
  sealed?: number;
  /** Wishes on the pair today (they weight tonight's list). */
  wishes: number;
  /** Whether this viewer already wished for it today. */
  wished: boolean;
}

/** One fusable pair on a run's board, as the forge in the shop shows it
 * (GET /runs/:id/forge). */
export interface ForgeSlot extends ForgeView {
  /** Board slots of the two units (the fuse's first and second). */
  first: number;
  second: number;
  /** A ready pair this run holds: the three designs to claim from. */
  candidates?: FusionDesign[];
}

/** A shop offer that brings a designed fusion within reach (GET
 * /runs/:id/forge's `offers`, #1049 reopened): buying `slot` gives a unit
 * that pairs with the Awoken unit at board slot `partner`, and their pair
 * has a design (discovered or ready to claim; unforged pairs never show).
 * `awakens`: buying it makes the third copy, so the fusion is one buy away;
 * false: you'd still need to awaken it (a quieter mark). */
export interface ForgeOffer {
  slot: number;
  partner: number;
  a: UnitId;
  b: UnitId;
  state: "discovered" | "ready";
  /** discovered: the canonical design (its name and emoji label the mark). */
  design?: FusionDesign;
  awakens: boolean;
}

/** What the forge of a run returns (GET /runs/:id/forge). */
export interface RunForge {
  pairs: ForgeSlot[];
  offers: ForgeOffer[];
}

/** The order-free key of a pair: "a+b" with the ids sorted. */
export function pairKey(x: UnitId, y: UnitId): string {
  const [a, b] = sortPair(x, y);
  return `${a}+${b}`;
}

export function sortPair(x: UnitId, y: UnitId): [UnitId, UnitId] {
  return x <= y ? [x, y] : [y, x];
}

/** An empty (unforged) pair. */
export function codexPair(x: UnitId, y: UnitId): CodexPair {
  const [a, b] = sortPair(x, y);
  return { a, b, candidates: [] };
}

export function pairState(p: CodexPair | undefined): PairState {
  if (!p || p.candidates.length === 0) return "unforged";
  return p.claim ? "discovered" : "ready";
}

/** The canonical design of a discovered pair. */
export function canonicalDesign(p: CodexPair | undefined): FusionDesign | undefined {
  return p?.claim ? p.candidates[p.claim.pick] : undefined;
}

export function forgeView(x: UnitId, y: UnitId, p: CodexPair | undefined, wishes: number, wished: boolean): ForgeView {
  const [a, b] = sortPair(x, y);
  const state = pairState(p);
  const view: ForgeView = { a, b, state, wishes, wished };
  if (state === "discovered") {
    view.design = canonicalDesign(p)!;
    view.claimedBy = p!.claim!.by;
    view.claimedAt = p!.claim!.at;
  } else if (state === "ready") view.sealed = p!.candidates.length;
  return view;
}

/** Why a claim of `pick` on `p` can't go through; null when it can. */
export function claimCheck(p: CodexPair | undefined, pick: number | undefined): string | null {
  const state = pairState(p);
  if (state === "unforged") return "not forged yet: wish for it";
  if (state === "discovered") return "already claimed";
  if (pick === undefined) return "pick one of the sealed designs";
  if (!Number.isInteger(pick) || pick < 0 || pick >= p!.candidates.length) return `no sealed design ${pick} (there are ${p!.candidates.length})`;
  return null;
}

/** Fuse two Awoken units into `design`: PWR and HP by fusedStats (the
 * stronger parent plus half the other), summed copies, the design's rule. The unit keeps `first`'s uid and is final;
 * `fusion` records the pair in key order and the credit. */
export function fuseDesigned(first: LineUnit, second: LineUnit, design: FusionDesign, discoveredBy: PlayerRef | null): LineUnit {
  const why = fuseCheck(first, second);
  if (why) throw new Error(`can't fuse ${first.name} + ${second.name}: ${why}`);
  const [a, b] = sortPair(first.unitId, second.unitId);
  return {
    uid: first.uid,
    kind: "fused",
    unitId: first.unitId,
    name: design.name,
    emoji: design.emoji ?? `${first.emoji}${second.emoji}`,
    copies: first.copies + second.copies,
    form: "awoken",
    stats: fusedStats(first.stats, second.stats),
    recipe: structuredClone(design.recipe),
    fusion: {
      first: a,
      second: b,
      name: design.name,
      discoveredBy,
      ...(design.abilities && Object.keys(design.abilities).length ? { abilities: structuredClone(design.abilities) } : {}),
      ...(design.statuses && Object.keys(design.statuses).length ? { statuses: structuredClone(design.statuses) } : {}),
    },
  };
}
