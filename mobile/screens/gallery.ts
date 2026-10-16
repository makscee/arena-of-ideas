// `#gallery` (mission #1286, slice 1, #1291): every unit of the live pool as
// a piece at every size, then the states (awoken, fused, statuses, a counter
// ring, their side, an offer), so a change to the piece is judged and
// screenshot-tested from one link. `#gallery=L` (or M, S, XS) shows one size.
import type { MvpContent, UnitContent } from "../../src/mvp/contract";
import { fusedFace } from "../../src/mvp/face";
import { h, show } from "../ui/dom";
import { faceOfUnit, piece, type PieceOptions, type PieceSize, type PieceUnit } from "../ui/piece";

const SIZES: PieceSize[] = ["L", "M", "S", "XS"];

const asPiece = (u: UnitContent, extra: Partial<PieceUnit> = {}): PieceUnit => ({
  name: u.name,
  emoji: u.emoji,
  stats: { ...u.base },
  face: faceOfUnit(u),
  tier: u.tier,
  copies: 1,
  ...extra,
});

function cell(u: PieceUnit, o: PieceOptions): HTMLElement {
  const shelf = (o.statuses?.length ?? 0) > 0 && o.size !== "XS";
  return h("div", { class: `g-cell ${o.size}${shelf ? " with-shelf" : ""}` }, piece(u, o));
}

export function galleryScreen(content: MvpContent, only: PieceSize | null): void {
  const sizes = only ? [only] : SIZES;
  const byName = (n: string) => content.units.find((u) => u.name === n);
  const units = content.units;
  const tabs = h(
    "div",
    { class: "g-tabs" },
    ...[["all", "#gallery"], ...SIZES.map((s) => [s, `#gallery=${s}`])].map(([label, href]) => h("a", { href: href! }, label!)),
  );

  // the states, on the first units that show them best
  const a = byName("Hornet") ?? units[0]!;
  const b = byName("Sniper") ?? units[1] ?? a;
  const ringed = units.find((u) => u.face?.counter) ?? a;
  const states: [string, PieceUnit, Partial<PieceOptions>][] = [
    ["sleeping, 2 of 3 copies", asPiece(a, { copies: 2 }), {}],
    ["awoken ★", asPiece(b, { awoken: true, stats: { pwr: b.base.pwr + 2, hp: b.base.hp + 3 } }), {}],
    [`fused ✦ (${a.name} + ${b.name})`, asPiece(a, { name: `${a.name}${b.name.toLowerCase()}`, fused: { emoji: b.emoji }, face: fusedFace(faceOfUnit(a)), stats: { pwr: a.base.pwr + b.base.pwr, hp: a.base.hp + b.base.hp } }), {}],
    ["with statuses", asPiece(a), { statuses: [{ status: "Shield", stacks: 1 }, { status: "Strength", stacks: 2 }, { status: "Poison", stacks: 3 }] }],
    [`counter ring 2/${ringed.face?.counter?.max ?? 3}`, asPiece(ringed), { counter: 2, statuses: [{ status: "Charge", stacks: 2 }, { status: "Shield", stacks: 1 }] }],
    ["their side", asPiece(b), { side: "them" }],
    ["an offer: price, locked", asPiece(a), { side: "shop", price: 3, lock: true }],
  ];
  const stateRows = states.map(([label, u, o]) =>
    h("div", { class: "g-unit", "data-testid": `gallery-state` }, h("div", { class: "g-name" }, label), h("div", { class: "g-row" }, ...sizes.map((size) => cell(u, { size, ...o })))),
  );

  const unitRows = units.map((u) => {
    const f = faceOfUnit(u);
    const pick = `${f.when.icon} ${f.when.label} · ${f.headline.icon}${f.headline.amount ? " " + f.headline.amount : ""}${f.counter ? ` · ring /${f.counter.max}` : ""}`;
    return h(
      "div",
      { class: "g-unit", "data-testid": "gallery-unit", "data-unit": u.id },
      h("div", { class: "g-name" }, u.name, h("small", {}, pick)),
      h("div", { class: "g-row" }, ...sizes.map((size) => cell(asPiece(u), { size }))),
    );
  });

  show(
    h(
      "div",
      { class: "gallery", "data-testid": "gallery" },
      h("h1", {}, `Pieces · ${units.length} units`),
      h("div", { class: "dim small" }, "L shop · M battle · S phone battle · XS chips"),
      tabs,
      h("h2", {}, "States"),
      ...stateRows,
      h("h2", {}, "Units"),
      ...unitRows,
    ),
  );
}
