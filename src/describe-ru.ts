// Russian card text (M4-3): the same sentences describe.ts derives, from the
// same DSL data, in Russian words and Russian grammar. Not a translation of the
// English sentence: a second word table (every When, Who, Does and status) plus
// the rules that join them ("Удар: 1 Яд атакующему.", "Начало боя: 2 Щита
// всем союзникам."). Segments carry the same terms, refs and sides as the
// English ones, so the client draws and links them the same way.
//
// Grammar: a target after a status or damage takes the dative ("всем
// союзникам"), after a verb the accusative ("воскресить павшего союзника");
// a counted noun takes its form by Intl.PluralRules("ru") (1 Щит, 2 Щита,
// 5 Щитов). Display-only, like describe.ts.

import type { Ability, Amount, Condition, Effect, EventPattern, Selector, StatusDef, UnitFilter, When } from "./types.js";
import type { DescribeOpts, DescribeSegment, PartRef, Side } from "./describe.js";
import type { TermId } from "./glossary.js";

const seg = (text: string): DescribeSegment => ({ text });

const capitalize = (s: string): string => (s.length > 0 ? s[0]!.toUpperCase() + s.slice(1) : s);

const PLURAL = new Intl.PluralRules("ru");

/** A counted noun's three forms: 1 щит, 2 щита, 5 щитов. */
type Forms = readonly [one: string, few: string, many: string];

/** The form a number takes: "one" (1, 21), "few" (2–4, 22), else "many". */
export function ruCount(n: number, forms: Forms): string {
  const cat = PLURAL.select(n);
  return cat === "one" ? forms[0] : cat === "few" ? forms[1] : forms[2];
}

/** A shipped status in Russian: its name (nominative, as a label), the
 * accusative ("получает Силу") and its counted forms ("2 Силы"). A status
 * the table doesn't know (player-made) keeps its own name in every case. */
interface RuStatus {
  name: string;
  acc: string;
  count: Forms;
}

export const RU_STATUSES: Record<string, RuStatus> = {
  Strength: { name: "Сила", acc: "Силу", count: ["Сила", "Силы", "Сил"] },
  Vitality: { name: "Живучесть", acc: "Живучесть", count: ["Живучесть", "Живучести", "Живучести"] },
  Curse: { name: "Проклятие", acc: "Проклятие", count: ["Проклятие", "Проклятия", "Проклятий"] },
  Poison: { name: "Яд", acc: "Яд", count: ["Яд", "Яда", "Ядов"] },
  Shield: { name: "Щит", acc: "Щит", count: ["Щит", "Щита", "Щитов"] },
  Freeze: { name: "Заморозка", acc: "Заморозку", count: ["Заморозка", "Заморозки", "Заморозок"] },
  Blessing: { name: "Благословение", acc: "Благословение", count: ["Благословение", "Благословения", "Благословений"] },
};

const statusRu = (s: string): RuStatus => RU_STATUSES[s] ?? { name: s, acc: s, count: [s, s, s] };

/** A status's Russian name (its label), or its own name when it has none. */
export const ruStatusName = (s: string): string => statusRu(s).name;

/** Stats, as the card shows them: АТК (PWR), ОЗ (HP). */
export const RU_STAT: Record<"pwr" | "hp", string> = { pwr: "АТК", hp: "ОЗ" };

const DAMAGE: Forms = ["урон", "урона", "урона"];
const STACKS: Forms = ["заряд", "заряда", "зарядов"];

const statSeg = (stat: "pwr" | "hp"): DescribeSegment => ({ text: RU_STAT[stat], term: `stat:${stat}` });

/** A derived amount after "равный" (dative): "АТК", "числу зарядов". */
function amountDat(a: Amount): DescribeSegment[] {
  switch (a.kind) {
    case "const":
      return [seg(String(a.value))];
    case "stat":
      return [statSeg(a.stat)];
    case "level":
      return [seg("уровню")];
    case "stacks":
      return [seg("числу "), { text: "зарядов", term: "term:stacks" }];
  }
}

/** An amount after "на" (heal by): "1", "АТК", "число зарядов". */
function amountAcc(a: Amount): DescribeSegment[] {
  switch (a.kind) {
    case "const":
      return [seg(String(a.value))];
    case "stat":
      return [statSeg(a.stat)];
    case "level":
      return [seg("уровень")];
    case "stacks":
      return [seg("число "), { text: "зарядов", term: "term:stacks" }];
  }
}

/** "1 заряд" / "2 заряда", the word carrying its term. */
const stacksSegs = (n: number): DescribeSegment[] => [seg(`${n} `), { text: ruCount(n, STACKS), term: "term:stacks" }];

/** An HP amount: "1 ОЗ", or a derived one, "ОЗ, равными числу зарядов"
 * after "с" (with), "ОЗ, равные числу зарядов" as an object. */
const hpSegs = (a: Amount, after: "с" | "acc"): DescribeSegment[] =>
  a.kind === "const" ? [{ text: String(a.value), term: "stat:hp", amount: true }, seg(" "), statSeg("hp")] : [statSeg("hp"), seg(after === "с" ? ", равными " : ", равные "), ...amountDat(a)];

// ---------- When ----------

/** Each event, card-speak: the holder's own ("удар"), and after "перед"
 * for an interceptor ("перед ударом"). Said of another unit: eventOf. */
const EVENT: Record<"Strike" | "Hurt" | "Heal" | "Death" | "Summon", { own: string; before: string }> = {
  Strike: { own: "атака", before: "атакой" },
  Hurt: { own: "удар", before: "ударом" },
  Heal: { own: "лечение", before: "лечением" },
  Death: { own: "смерть", before: "смертью" },
  Summon: { own: "призыв", before: "призывом" },
};

/** Whose event, in the case each phrasing needs. */
const WHO: Record<Exclude<UnitFilter, "holder">, { nom: string; gen: string; dat: string }> = {
  ally: { nom: "союзник", gen: "союзника", dat: "союзнику" },
  otherAlly: { nom: "союзник", gen: "союзника", dat: "союзнику" },
  enemy: { nom: "враг", gen: "врага", dat: "врагу" },
  any: { nom: "любой", gen: "любого", dat: "любому" },
};

/** The event noun said of a unit: "атака союзника", "удар по врагу". */
function eventOf(on: keyof typeof EVENT, f: UnitFilter | undefined): string {
  if (f === "holder") return EVENT[on].own;
  const w = WHO[f ?? "any"];
  return on === "Hurt" ? `удар по ${w.dat}` : `${EVENT[on].own} ${w.gen}`;
}

/** describeWhenSegments in Russian: the same runs (the trigger's term, the
 * Part ref, `clause: "when"`, the scope; an interceptor's "перед" its own
 * `term:would` run; a status its own run). */
export function describeWhenSegmentsRu(w: When): DescribeSegment[] {
  const p: EventPattern = w.on;
  const intercept = w.kind === "interceptor";
  const ref: PartRef = { family: intercept ? "interceptor" : "trigger", kind: p.on };
  const term: TermId = `trigger:${p.on}`;
  const filter = p.on === "Strike" ? p.striker : "unit" in p ? p.unit : undefined;
  const scope: { scope?: UnitFilter } = p.on === "BattleStart" || p.on === "TurnStart" || p.on === "TurnEnd" ? {} : { scope: filter ?? "any" };
  const run = (text: string): DescribeSegment => ({ text, partRef: ref, term, clause: "when", ...scope });
  const would = (text: string): DescribeSegment => ({ text, partRef: ref, term: "term:would", clause: "when" });
  const statusSeg = (status: string): DescribeSegment => ({ text: statusRu(status).acc, statusRef: status, term: `status:${status}`, clause: "when" });
  const who = filter === undefined || filter === "holder" ? undefined : WHO[filter];
  switch (p.on) {
    case "BattleStart":
      return [run("начало боя")];
    case "TurnStart":
      return [run("начало хода")];
    case "TurnEnd":
      return [run("конец хода")];
    case "Strike":
    case "Hurt":
    case "Heal":
    case "Death":
    case "Summon": {
      if (!intercept) return [run(eventOf(p.on, filter))];
      // "перед ударом", "перед ударом по союзнику", "перед смертью врага".
      const rest = filter === "holder" ? "" : p.on === "Hurt" ? ` по ${WHO[filter ?? "any"].dat}` : ` ${WHO[filter ?? "any"].gen}`;
      return [would("перед"), run(` ${EVENT[p.on].before}${rest}`)];
    }
    case "StatusApplied":
    case "StatusRemoved": {
      const v = p.on === "StatusApplied" ? (intercept ? "получит" : "получает") : intercept ? "потеряет" : "теряет";
      const lead: DescribeSegment[] = intercept ? [would("вот-вот"), run(` ${who ? `${who.nom} ` : ""}${v}`)] : [run(`${who ? `${who.nom} ` : ""}${v}`)];
      if (p.status === undefined) {
        const last = lead[lead.length - 1]!;
        lead[lead.length - 1] = { ...last, text: `${last.text} статус` };
        return lead;
      }
      const last = lead[lead.length - 1]!;
      lead[lead.length - 1] = { ...last, text: `${last.text} ` };
      return [...lead, statusSeg(p.status)];
    }
    case "StatChanged": {
      const stat = p.stat !== undefined ? RU_STAT[p.stat] : "показатель";
      if (p.sign === "gain" || p.sign === "loss") return [run(`${who ? `${who.nom} ` : ""}${p.sign === "gain" ? "набирает" : "теряет"} ${stat}`)];
      return [run(`${stat}${who ? ` ${who.gen}` : ""} меняется`)];
    }
  }
}

// ---------- Who ----------

/** A selector in the two cases the sentences need: dative after a status or
 * damage ("всем врагам"), accusative after a verb ("павшего союзника"). */
const SELECTOR: Record<Exclude<Selector["kind"], "holder" | "eventUnit">, { dat: string; acc: string }> = {
  attacker: { dat: "атакующему", acc: "атакующего" },
  frontEnemy: { dat: "переднему врагу", acc: "переднего врага" },
  allEnemies: { dat: "всем врагам", acc: "всех врагов" },
  allAllies: { dat: "всем союзникам", acc: "всех союзников" },
  randomEnemy: { dat: "случайному врагу", acc: "случайного врага" },
  lastDeadAlly: { dat: "павшему союзнику", acc: "павшего союзника" },
};

/** The holder: "себе"/"себя" on a unit's own ability, "носителю"/"носителя"
 * on a status's (opts.holder "holder"). */
const holderCase = (opts: DescribeOpts, c: Case): string => (opts.holder === "holder" ? (c === "dat" ? "носителю" : "носителя") : c === "dat" ? "себе" : "себя");

type Case = "dat" | "acc";

const SELECTOR_SIDE: Record<Selector["kind"], Side | undefined> = {
  holder: "ally",
  eventUnit: undefined,
  attacker: "enemy",
  frontEnemy: "enemy",
  allEnemies: "enemy",
  allAllies: "ally",
  randomEnemy: "enemy",
  lastDeadAlly: "ally",
};

function selectorText(s: Selector, c: Case, opts: DescribeOpts): string {
  switch (s.kind) {
    case "holder":
      return holderCase(opts, c);
    case "eventUnit":
      // "it": the unit the trigger is about; the holder's own event reads as the holder.
      return opts.eventUnit?.text === (opts.holder ?? "self") ? holderCase(opts, c) : c === "dat" ? "ему" : "его";
    default:
      return SELECTOR[s.kind][c];
  }
}

function targetSegs(selectors: Selector[], c: Case, opts: DescribeOpts): DescribeSegment[] {
  const out: DescribeSegment[] = [];
  selectors.forEach((s, i) => {
    if (i > 0) out.push(seg(" и "));
    const side = s.kind === "eventUnit" ? opts.eventUnit?.side : SELECTOR_SIDE[s.kind];
    out.push({ text: selectorText(s, c, opts), partRef: { family: "selector", kind: s.kind }, term: `target:${s.kind}`, ...(side ? { side } : {}) });
  });
  return out;
}

// ---------- Does ----------

/** "1 урон", "2 Щита": the head of an effect that reads "amount, word, to T". */
function shortHead(e: Effect): DescribeSegment[] | undefined {
  const ref: PartRef = { family: "effect", kind: e.kind };
  if (e.kind === "damage" && e.amount.kind === "const")
    return [
      { text: String(e.amount.value), partRef: ref, term: "effect:damage", amount: true },
      { text: " ", partRef: ref },
      { text: ruCount(e.amount.value, DAMAGE), partRef: ref, term: "effect:damage" },
    ];
  if (e.kind === "applyStatus" && e.stacks.kind === "const")
    return [
      { text: String(e.stacks.value), partRef: ref, term: `status:${e.status}`, amount: true },
      { text: " ", partRef: ref },
      { text: ruCount(e.stacks.value, statusRu(e.status).count), statusRef: e.status, term: `status:${e.status}` },
    ];
  return undefined;
}

/** The event an ability's "cancel it" cancels, by its noun's gender: атаку
 * → "её", удар → "его". */
const CANCEL_IT: Partial<Record<EventPattern["on"], string>> = { Strike: "её", Death: "её" };

interface EffectCtx extends DescribeOpts {
  selectors: Selector[];
  event?: EventPattern["on"];
}

function effectSegs(e: Effect, ctx: EffectCtx): DescribeSegment[] {
  const ref: PartRef = { family: "effect", kind: e.kind };
  const e0 = (text: string): DescribeSegment => ({ text, partRef: ref });
  const eT = (text: string): DescribeSegment => ({ text, partRef: ref, term: `effect:${e.kind}` });
  const statusSeg = (status: string, text: string): DescribeSegment => ({ text, statusRef: status, term: `status:${status}` });
  const dat = targetSegs(ctx.selectors, "dat", ctx);
  const acc = targetSegs(ctx.selectors, "acc", ctx);
  const head = shortHead(e);
  if (head) return [...head, e0(" "), ...dat];
  switch (e.kind) {
    case "damage":
      // "урон, равный АТК, переднему врагу".
      return [eT("урон"), e0(", равный "), ...amountDat(e.amount), e0(", "), ...dat];
    case "heal":
      // "лечение на 1 всем союзникам".
      return e.amount.kind === "const"
        ? [eT("лечение"), e0(" на "), { ...eT(String(e.amount.value)), amount: true }, e0(" "), ...dat]
        : [eT("лечение"), e0(" на "), ...amountAcc(e.amount), e0(" "), ...dat];
    case "applyStatus":
      return [statusSeg(e.status, statusRu(e.status).name), e0(", равная "), ...amountDat(e.stacks), e0(", "), ...dat];
    case "consumeStacks": {
      if (e.stacks.kind === "const")
        return e.status !== undefined
          ? [e0(`потратить ${e.stacks.value} `), statusSeg(e.status, ruCount(e.stacks.value, statusRu(e.status).count))]
          : [e0("потратить "), ...stacksSegs(e.stacks.value)];
      const which: DescribeSegment = e.status !== undefined ? statusSeg(e.status, statusRu(e.status).name) : { text: "заряды", term: "term:stacks" };
      return [e0("потратить "), which, e0(" в количестве, равном "), ...amountDat(e.stacks)];
    }
    case "summon": {
      // Unit names stay as they are (M4-4 translates them).
      const unitRun = (side: Side): DescribeSegment => ({
        text: `${e.unit.name} (${e.unit.base.pwr}/${e.unit.base.hp})`,
        partRef: ref,
        unitRef: e.unit.name.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
        side,
      });
      const summon = (side: Side): DescribeSegment[] => [eT("призвать "), unitRun(side)];
      const kinds = ctx.selectors.map((s) => s.kind);
      const side: Side = acc.find((t) => t.side)?.side ?? "ally";
      if (kinds.length === 1 && (kinds[0] === "allAllies" || kinds[0] === "allEnemies")) {
        const ally = kinds[0] === "allAllies";
        const each = acc.map((t) => (t.partRef?.family === "selector" ? { ...t, text: ally ? "каждого союзника" : "каждого врага" } : t));
        return [...summon(ally ? "ally" : "enemy"), e0(" за "), ...each, e0(`${ally && !ctx.holderGone ? `, включая ${holderCase(ctx, "acc")}` : ""}, если есть место`)];
      }
      if (kinds.length === 1 && kinds[0] === "holder") return summon("ally");
      return [...summon(side), e0(" за "), ...acc];
    }
    case "silence":
      return [eT("заглушить"), e0(" "), ...acc];
    case "resurrect":
      return [eT("воскресить"), e0(" "), ...acc, e0(" с "), ...hpSegs(e.hp, "с")];
    case "cancel": {
      const it = (ctx.event && CANCEL_IT[ctx.event]) ?? "его";
      return e.consumeSelf !== undefined ? [eT(`отменить ${it}`), e0(", потратить "), ...stacksSegs(e.consumeSelf)] : [eT(`отменить ${it}`)];
    }
    case "absorbHurt":
      return [eT("заблокировать урон"), e0(" до числа "), { text: "зарядов", term: "term:stacks" }, e0(", тратя их")];
    case "preventDeathHeal":
      return [eT("отменить смерть"), e0(", установить "), ...dat, e0(" "), ...hpSegs(e.toHp, "acc"), e0(e.removeSelf ? ", потратить этот статус" : "")];
  }
}

function describeConditionSegmentsRu(c: Condition): DescribeSegment[] {
  const partRef: PartRef = { family: "condition", kind: c.kind };
  switch (c.kind) {
    case "holderHpAtMost":
      return [{ text: `при ${c.value} ОЗ или меньше`, partRef, term: `condition:${c.kind}` }];
  }
}

/** describeAbilitySegments in Russian. `opts.eventUnit` is worked out by
 * describe.ts before it hands over. */
export function describeAbilitySegmentsRu(ab: Ability, opts: DescribeOpts): DescribeSegment[] {
  const segs: DescribeSegment[] = [];
  (ab.whens ?? []).forEach((w, i) => {
    if (i > 0) segs.push(seg(", или "));
    segs.push(...describeWhenSegmentsRu(w));
  });
  if (segs.length > 0) segs[0] = { ...segs[0]!, text: capitalize(segs[0]!.text) };
  if (ab.condition !== undefined) segs.push(seg(", "), ...describeConditionSegmentsRu(ab.condition));
  segs.push(seg(": "));
  const gone = (ab.whens ?? []).some((w) => w.kind !== "interceptor" && w.on.on === "Death" && w.on.unit === "holder");
  const ctx: EffectCtx = { ...opts, selectors: ab.selectors ?? [], ...(gone ? { holderGone: true } : {}), ...(ab.whens?.[0] ? { event: ab.whens[0].on.on } : {}) };
  // The same merging as English: a run of "amount, word, to T" effects names T
  // once, joined by commas and a final "и"; any other pair keeps ", затем".
  ab.effects.forEach((e, i) => {
    const head = shortHead(e);
    const nextHead = i + 1 < ab.effects.length && head !== undefined && shortHead(ab.effects[i + 1]!) !== undefined;
    const prevHead = i > 0 && head !== undefined && shortHead(ab.effects[i - 1]!) !== undefined;
    if (i > 0) segs.push(seg(prevHead ? (nextHead ? ", " : " и ") : ", затем "));
    if (nextHead) segs.push(...head!);
    else segs.push(...effectSegs(e, ctx));
  });
  segs.push(seg("."));
  return segs;
}

/** describeStatusSegments in Russian: "+1 АТК за заряд.", then each ability
 * said of its holder ("носителю"). */
export function describeStatusSegmentsRu(def: StatusDef): DescribeSegment[] {
  const segs: DescribeSegment[] = [];
  if (def.statMods !== undefined) {
    for (const stat of ["hp", "pwr"] as const) {
      const v = def.statMods[stat];
      if (v === undefined || v === 0) continue;
      if (segs.length > 0) segs.push(seg(", "));
      segs.push(seg(`${v > 0 ? "+" : ""}${v} `), statSeg(stat), seg(" за "), { text: "заряд", term: "term:stacks" });
    }
    if (segs.length > 0) segs.push(seg("."));
  }
  for (const action of def.abilities) {
    if (segs.length > 0) segs.push(seg(" "));
    const ab: Ability = {
      ...action,
      whens: def.triggers ?? action.whens ?? [],
      selectors: def.selectors ?? action.selectors ?? [],
      ...(def.condition ?? action.condition ? { condition: def.condition ?? action.condition } : {}),
    };
    segs.push(...describeAbilitySegmentsRu(ab, { holder: "holder", lang: "ru" }));
  }
  if (segs.length === 0) segs.push(seg("Нет эффекта."));
  return segs;
}
