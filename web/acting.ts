// The acting-card battle presentation (#082 slice D) — the mockup's "compact
// board + acting full card". Pure projection over the kernel log: every fact
// (acting unit, target, the step's effects, the reactive chains, the trace
// chips, the per-side ACTING/TARGET/USED state) is DERIVED from structured
// events (Strike's striker/defender, each caused event's `source`/`causedBy`),
// never regex over narrated text. DOM-free — the viewer owns the playhead and
// injects the returned HTML; this file owns the mapping and the markup.

import {
  FAMILY_HEX,
  abilityChips,
  beatAtStep,
  type Ability,
  type AbilityRef,
  type AbilityRegistry,
  type Beat,
  type BattleEvent,
  type Family,
  type NameOf,
  type Side,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";
import { nameFamily } from "./unit-card.js";
import { abilityStar, actionIcon, triggerIcon, triggerIconForGlyph } from "./glyphs.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

// Trigger label per event-pattern `on` — mirrors describe.ts's TRIGGER_CHIP
// labels (the chain callout says "ON DAMAGED", "ON STRIKE", …). The mark is
// always the burst (a reactive chain — the mockup's chain trigger mark).
const TRIGGER_LABEL: Record<string, string> = {
  BattleStart: "On battle start",
  TurnStart: "On turn start",
  TurnEnd: "On turn end",
  Strike: "On strike",
  Hurt: "On damaged",
  Heal: "On heal",
  Death: "On death",
  Summon: "On summon",
  StatusApplied: "Status gained",
  StatusRemoved: "Status lost",
};

/** What context the viewer hands the pure model — the unit defs (for family +
 * ability) and the ability/status registries the refs resolve through. */
export interface ActingCtx {
  defs: Map<string, UnitDef>;
  abilities: AbilityRegistry;
  registry: StatusRegistry;
  name: NameOf;
  sideOf: (id: string) => Side | undefined;
}

/** The acting unit + its target for the beat the playhead sits in. A Strike
 * beat names both from structured ids (striker acts, defender is hit); every
 * other root kind (BattleStart, TurnStart/End, Fatigue, PairFaced, BattleEnd)
 * has no single actor — the centre shows a phase caption instead. */
export function actingUnitAt(beats: Beat[], step: number): { acting?: string; target?: string } {
  const at = beatAtStep(beats, step);
  if (!at) return {};
  const root = at.beat.root;
  if (root.type === "Strike") return { acting: root.striker, target: root.defender };
  return {};
}

/** Units that already STRUCK earlier this turn (a Strike beat that opened before
 * the current beat, same turn) — dimmed + struck-through on their side card. The
 * unit acting now is never "used" (the caller excludes it). */
export function usedThisTurnAt(log: BattleEvent[], beats: Beat[], step: number): Set<string> {
  const used = new Set<string>();
  const at = beatAtStep(beats, step);
  if (!at) return used;
  const turn = at.beat.root.turn;
  for (const b of beats) {
    if (b.start >= at.beat.start) break; // only beats strictly before the current one
    if (b.root.type === "Strike" && b.root.turn === turn) used.add(b.root.striker);
  }
  return used;
}

/** A unit's colour family — its ability's family (PRD #081), degrading to a
 * stable name→family when the ability isn't in the registry (so a card still
 * paints coloured). */
export function familyOf(ctx: ActingCtx, unitId: string): Family {
  const def = ctx.defs.get(unitId);
  const ab = def !== undefined ? ctx.abilities[def.ability] : undefined;
  return ab?.family ?? nameFamily(def?.name ?? unitId);
}

/** The terse cap-label + chip segments for a unit's ability, with the live
 * target name folded into the target slot (so the card reads "⚔ Brawler ▸ ☣
 * Poison 2", not "⚔ Front enemy …"). */
export function abilityLineFor(
  ctx: ActingCtx,
  unitId: string,
  targetName?: string,
): {
  abilityLabel?: string | undefined;
  trigger?: string | undefined;
  triggerGlyph?: string | undefined;
  target?: string | undefined;
  action?: string | undefined;
} {
  const def = ctx.defs.get(unitId);
  const ab = def !== undefined ? ctx.abilities[def.ability] : undefined;
  if (ab === undefined) return {};
  const chips = abilityChips(ab);
  return {
    abilityLabel: ab.name,
    ...chips,
    // The live defender is more specific than the selector chip ("Front enemy").
    ...(targetName !== undefined ? { target: targetName } : {}),
  };
}

export interface ResultRow {
  id: number; // the caused event this row narrates (clickable → cause trace)
  glyph: string; // an inline SVG mark (#086), drawn in the row's `cls` colour
  cls: string; // family/colour class on the glyph
  html: string; // the row text, pre-escaped + tinted spans
}

export interface ChainBox {
  unit: string;
  trigger: string; // "ON DAMAGED"
  rows: ResultRow[];
}

export interface ActingModel {
  kind: "card" | "phase";
  beatIndex: number; // the beat the playhead sits in — stable across the beat's steps
  triggerIndex: number; // the playhead's 1-based index (#42, matches the scrubber)
  // card:
  acting?: {
    id: string;
    name: string;
    side: Side | undefined;
    family: Family;
    hex: string;
    abilityLabel: string;
    now: { trigGlyph: string; targetName?: string; action?: string };
  };
  result: ResultRow[];
  chains: ChainBox[];
  // phase:
  caption?: string;
}

/** The phase caption for a beat with no acting unit. */
function phaseCaption(root: BattleEvent): string {
  switch (root.type) {
    case "BattleStart":
      return "Battle begins";
    case "TurnStart":
      return `Turn ${root.turn} begins`;
    case "TurnEnd":
      return "End of turn";
    case "Fatigue":
      return "Fatigue";
    case "PairFaced":
      return "The coin flips";
    case "BattleEnd":
      return root.winner === "draw" ? "Draw" : `Side ${root.winner} wins`;
    default:
      return root.type;
  }
}

/** Resolve a caused event's source ability to its trigger's `on` kind — a unit
 * ability via the unit's def + registry, a status ability via the status
 * registry. Undefined when it can't be resolved (kept honest — the chain box
 * then just omits the trigger label). */
function triggerOf(ctx: ActingCtx, ref: AbilityRef): string | undefined {
  let ab: Ability | undefined;
  if (ref.status !== undefined) {
    ab = ctx.registry[ref.status]?.abilities[ref.ability];
  } else {
    const def = ctx.defs.get(ref.unit);
    ab = def !== undefined ? ctx.abilities[def.ability] : undefined;
  }
  return ab?.whens[0]?.on.on;
}

/** A caused hero-effect event → a RESULT/CHAIN row (glyph + tinted text). The
 * `family` colours an ability-applied status glyph; kernel damage stays red. */
function effectRow(ctx: ActingCtx, e: BattleEvent, family: Family | undefined): ResultRow | undefined {
  const who = (id: string): string => {
    const side = ctx.sideOf(id);
    const cls = side === "A" ? "u ua" : side === "B" ? "u ub" : "u";
    return `<span class="${cls}">${esc(ctx.name(id))}</span>`;
  };
  // The row glyph is an inline SVG mark (#086) — the vendored fonts lack these
  // unicode marks; a status the ability applies wears the ability FAMILY's mark
  // (◆ poison, etc.) exactly as the card's action chip does.
  const famGlyph = family !== undefined ? actionIcon(family) : abilityStar();
  switch (e.type) {
    case "Hurt": {
      const hp = e.hpAfter === undefined ? "" : ` → <b>${Math.max(0, e.hpAfter)}</b> HP`;
      const abs = e.absorbed !== undefined ? ` <span class="r-dim">(${e.absorbed} absorbed)</span>` : "";
      return { id: e.id, glyph: actionIcon("damage"), cls: "r-hurt", html: `${who(e.unit)} takes <b class="r-hurt">${e.amount}</b>${abs}${hp}` };
    }
    case "Heal": {
      const hp = e.hpAfter === undefined ? "" : ` → <b>${Math.max(0, e.hpAfter)}</b> HP`;
      return { id: e.id, glyph: actionIcon("heal"), cls: "r-heal", html: `${who(e.unit)} heals <b class="r-heal">${e.amount}</b>${hp}` };
    }
    case "StatusApplied":
      return {
        id: e.id,
        glyph: famGlyph,
        cls: "r-status",
        html: `${who(e.unit)} gains <b class="r-status">${esc(e.status)} ${e.stacks}</b>`,
      };
    case "StatusRemoved":
      return {
        id: e.id,
        glyph: actionIcon("status-loss"),
        cls: "r-status",
        html:
          e.remaining > 0
            ? `${who(e.unit)} loses <b>${e.stacks}</b> ${esc(e.status)} <span class="r-dim">(${e.remaining} left)</span>`
            : `${who(e.unit)}'s ${esc(e.status)} <span class="r-dim">spent</span>`,
      };
    case "StatChanged": {
      const up = e.delta >= 0;
      const statName = e.stat === "pwr" ? "Power" : "HP";
      return {
        id: e.id,
        glyph: actionIcon(up ? "stat-up" : "stat-down"),
        cls: up ? "r-heal" : "r-hurt",
        html: `${who(e.unit)} gains <b class="${up ? "r-heal" : "r-hurt"}">${up ? "+" : ""}${e.delta} ${statName}</b>`,
      };
    }
    case "Death":
      return { id: e.id, glyph: triggerIcon("death"), cls: "r-death", html: `${who(e.unit)} <b class="r-death">dies</b>` };
    case "Summon":
      return {
        id: e.id,
        glyph: actionIcon("summon"),
        cls: "r-summon",
        html: e.resurrected
          ? `${who(e.unit)} <b>rises</b> at ${e.atHp ?? 1} HP`
          : `${who(e.unit)} is <b>summoned</b> (${e.hp} HP)`,
      };
    case "Silenced":
      return { id: e.id, glyph: actionIcon("silence"), cls: "r-warn", html: `${who(e.unit)} is <b class="r-warn">silenced</b>` };
    default:
      return undefined;
  }
}

/** The centre acting-card model for the playhead at `step` — a card for the
 * acting unit (Strike beat) or a phase caption otherwise. RESULT holds the
 * beat's DIRECT effects revealed so far (source = kernel, or the acting unit's
 * own ability); CHAINS holds the REACTIVE effects (a DIFFERENT unit's ability
 * triggered by this step), grouped by the chaining unit. The kernel `source`
 * field is the structured handle that splits the two — never a guess. */
export function actingModelAt(log: BattleEvent[], beats: Beat[], step: number, ctx: ActingCtx): ActingModel {
  const at = beatAtStep(beats, step);
  const triggerIndex = step + 1;
  if (!at) return { kind: "phase", beatIndex: -1, triggerIndex, result: [], chains: [], caption: "—" };
  const { beat } = at;
  const { acting, target } = actingUnitAt(beats, step);

  if (acting === undefined) {
    return { kind: "phase", beatIndex: beat.index, triggerIndex, result: [], chains: [], caption: phaseCaption(beat.root) };
  }

  const actFamily = familyOf(ctx, acting);
  const line = abilityLineFor(ctx, acting, target !== undefined ? ctx.name(target) : undefined);

  // Caused hero events revealed so far, split by source into DIRECT vs REACTIVE.
  const revealed = beat.caused.filter((e) => e.id <= step);
  const result: ResultRow[] = [];
  const chainGroups = new Map<string, ChainBox>();
  for (const e of revealed) {
    const reactive = e.source !== "kernel" && e.source.unit !== acting;
    if (!reactive) {
      // Direct: kernel consequence OR the acting unit's own ability.
      const fam = e.source !== "kernel" ? familyOf(ctx, e.source.unit) : actFamily;
      const row = effectRow(ctx, e, fam);
      if (row) result.push(row);
    } else {
      const ref = e.source as AbilityRef;
      const key = `${ref.unit}|${ref.status ?? ""}|${ref.ability}`;
      let box = chainGroups.get(key);
      if (box === undefined) {
        const on = triggerOf(ctx, ref);
        const trig = on !== undefined ? (TRIGGER_LABEL[on] ?? on).toUpperCase() : "TRIGGERED";
        box = { unit: ctx.name(ref.unit), trigger: trig, rows: [] };
        chainGroups.set(key, box);
      }
      const row = effectRow(ctx, e, familyOf(ctx, ref.unit));
      if (row) box.rows.push(row);
    }
  }

  return {
    kind: "card",
    beatIndex: beat.index,
    triggerIndex,
    acting: {
      id: acting,
      name: ctx.name(acting),
      side: ctx.sideOf(acting),
      family: actFamily,
      hex: FAMILY_HEX[actFamily],
      abilityLabel: line.abilityLabel ?? actFamily.toUpperCase(),
      now: {
        trigGlyph: line.triggerGlyph ?? "⚔",
        ...(line.target !== undefined ? { targetName: line.target } : {}),
        ...(line.action !== undefined ? { action: line.action } : {}),
      },
    },
    result,
    chains: [...chainGroups.values()].filter((c) => c.rows.length > 0),
  };
}

// ---------- render: the centre acting card ----------

function rowHtml(r: ResultRow): string {
  return `<div class="ac-row" data-id="${r.id}"><span class="ac-g ${r.cls}">${r.glyph}</span><span class="ac-t">${r.html}</span></div>`;
}

/** The centre slot HTML for `model` — the big acting card (family border +
 * glow, header sigil + named ability + #index, the ● NOW chip, RESULT rows and
 * any ↳ CHAINS callouts) or, for a beat with no actor, a centred phase caption.
 * `sigil` is the family sigil SVG the caller built (unit-card's familySigil). */
export function actingCardHtml(model: ActingModel, sigil: string): string {
  if (model.kind === "phase") {
    return `<div class="acting-phase" data-beat="${model.beatIndex}"><span class="ap-cap">${esc(model.caption ?? "")}</span></div>`;
  }
  const a = model.acting!;
  const sideCls = a.side === "A" ? "u ua" : a.side === "B" ? "u ub" : "u";
  const now = a.now;
  const nowSegs: string[] = [];
  if (now.targetName !== undefined)
    nowSegs.push(`<span class="ac-now-tgt">${triggerIconForGlyph(now.trigGlyph)} ${esc(now.targetName)}</span>`);
  if (now.action !== undefined)
    nowSegs.push(`<span class="ac-now-act">${actionIcon(a.family)} ${esc(now.action)}</span>`);
  const nowRow =
    nowSegs.length > 0
      ? `<div class="ac-now"><span class="ac-now-k">● NOW</span>${nowSegs.join('<span class="ac-arrow">▸</span>')}</div>`
      : "";

  const resultRows = model.result.map(rowHtml).join("");
  const chains = model.chains
    .map(
      (c) =>
        `<div class="ac-chain"><div class="ac-chain-h">↳ CHAINS · ${esc(c.unit.toUpperCase())} · ${triggerIcon("damaged")} ${esc(c.trigger)}</div>${c.rows
          .map(rowHtml)
          .join("")}</div>`,
    )
    .join("");
  const result =
    resultRows !== "" || chains !== ""
      ? `<div class="ac-result"><div class="ac-result-k">RESULT</div>${resultRows}${chains}</div>`
      : "";

  return `
    <div class="acting-card" style="--fam:${a.hex}" data-acting="${esc(a.id)}" data-beat="${model.beatIndex}">
      <div class="ac-head">
        <div class="ac-sigil">${sigil}</div>
        <div class="ac-id">
          <div class="ac-name ${sideCls}">${esc(a.name)}</div>
          <div class="ac-ability">${abilityStar("ac-spark")}<span>${esc(a.abilityLabel.toUpperCase())}</span></div>
        </div>
        <div class="ac-idx">#${model.triggerIndex}</div>
      </div>
      ${nowRow}
      ${result}
    </div>`;
}

// ---------- render: the bottom trace strip ----------

export interface TraceChip {
  id: number; // the root event id this chip scrubs to (its beat.end, fully revealed)
  primary: string; // "TXS · VENOMANCER" or "END OF TURN"
  secondary: string; // a terse effect summary — pre-built HTML (an inline glyph + escaped text)
  family?: Family;
  current: boolean;
}

/** A 3-letter ability abbreviation for the trace chip (mockup TXS·VENOMANCER). */
function abbrev(s: string): string {
  const letters = s.replace(/[^a-z]/gi, "").toUpperCase();
  return letters.slice(0, 3) || "···";
}

/** A terse one-line effect summary for a beat's trace chip. */
function beatSummary(ctx: ActingCtx, beat: Beat): string {
  // The most salient caused hero effect: a status applied, else damage, else
  // the phase's own word.
  for (const e of beat.caused) {
    if (e.type === "StatusApplied") return `${actionIcon("poison")} ${esc(e.status)} ${e.stacks}`;
    if (e.type === "Death") return `${triggerIcon("death")} ${esc(ctx.name(e.unit))} dies`;
  }
  for (const e of beat.caused) {
    if (e.type === "Hurt") return `${actionIcon("damage")} ${esc(ctx.name(e.unit))} −${e.amount}`;
  }
  if (beat.kind === "TurnEnd") return "end ticks";
  if (beat.kind === "Fatigue") return "everyone worn";
  return "";
}

/** One trace chip per beat — the bottom strip. The chip the playhead sits in is
 * `current` (highlighted + "now"); clicking any chip scrubs to that beat. */
export function traceChipsAt(log: BattleEvent[], beats: Beat[], step: number, ctx: ActingCtx): TraceChip[] {
  const cur = beatAtStep(beats, step)?.beat.index;
  return beats.map((beat) => {
    let primary: string;
    let family: Family | undefined;
    if (beat.root.type === "Strike") {
      const def = ctx.defs.get(beat.root.striker);
      const ab = def !== undefined ? ctx.abilities[def.ability] : undefined;
      primary = `${abbrev(ab?.name ?? def?.name ?? "act")} · ${ctx.name(beat.root.striker).toUpperCase()}`;
      family = familyOf(ctx, beat.root.striker);
    } else {
      primary = phaseCaption(beat.root).toUpperCase();
    }
    return {
      id: beat.end,
      primary,
      secondary: beatSummary(ctx, beat),
      ...(family !== undefined ? { family } : {}),
      current: beat.index === cur,
    };
  });
}

/** The horizontal trace strip — a clickable chip per beat, the current one lit.
 * Each chip carries `data-id` = the beat-end event to scrub to (preserving the
 * old log-row scrub/cause-select behaviour, re-presented as the strip). */
export function traceStripHtml(chips: TraceChip[]): string {
  const cells = chips
    .map((c) => {
      const fam = c.family !== undefined ? FAMILY_HEX[c.family] : "";
      const style = fam !== "" ? ` style="--fam:${fam}"` : "";
      const cls = ["tr-chip", c.current ? "is-cur" : "", c.family !== undefined ? "has-fam" : ""].filter(Boolean).join(" ");
      const now = c.current ? ' <span class="tr-now">◂ now</span>' : "";
      return `<button type="button" class="${cls}"${style} data-id="${c.id}" title="${esc(c.primary)}"><span class="tr-p">${esc(c.primary)}${now}</span><span class="tr-s">${c.secondary}</span></button>`;
    })
    .join("");
  return `<div class="trace-strip" role="list">${cells}</div>`;
}
