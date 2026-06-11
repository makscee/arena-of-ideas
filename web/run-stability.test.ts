// Slice 3 stability invariants — layout tests that run without a browser.
//
// Tests fall into three groups:
//  1. Ladder accordion: expanded pool body is wrapped in .lv-pool-body so the
//     CSS max-height cap applies (LS-4).
//  2. Notice/error DOM: the notice and error elements never get the `hidden`
//     attribute set — they stay in flow and reserve space via min-height (LS-5).
//  3. Shop row/line min-height: the CSS sets min-height on #run-shop-row and
//     #run-line; this test acts as a canary — if someone removes the reserved
//     height the Playwright check catches it, but the unit test documents the
//     intent (LS-6).

import { describe, expect, test } from "vitest";
import {
  InMemoryLadderStore,
  buy,
  initRun,
  ladderFight,
  stressRegistry,
  type TeamSnapshot,
} from "../src/index.js";
import { createLadderView } from "./ladder-view.js";

// Minimal DOM stub — enough for innerHTML assignment and querySelector.
function fakeRoot(): HTMLElement {
  let _html = "";
  const el = {
    get innerHTML() {
      return _html;
    },
    set innerHTML(v: string) {
      _html = v;
    },
    querySelector(sel: string): Element | null {
      // Only used inside render() to find anchor for inspector.
      // Tests that don't open an inspector can return null safely.
      void sel;
      return null;
    },
    addEventListener(_type: string, _fn: EventListener) {},
  } as unknown as HTMLElement;
  return el;
}

// Seed two teams from the built-in pool so the ladder has real ghosts.
function buildLadder(): InMemoryLadderStore {
  const store = new InMemoryLadderStore();
  for (let seed = 0; seed < 2; seed++) {
    let s = initRun({ seed, runId: `run-${seed}`, pool: [{ name: "Titan", base: { hp: 100, pwr: 50 } }], statuses: stressRegistry });
    s = buy(s, 0);
    while (s.status === "active") s = ladderFight(s, store);
  }
  return store;
}

describe("ladder accordion pool-body wrapper", () => {
  test("collapsed rounds have no .lv-pool-body", () => {
    const store = buildLadder();
    const root = fakeRoot();
    const view = createLadderView(root, { store, registry: stressRegistry });
    view.refresh({ round: 1, runId: "run-0" });
    // No round is expanded on first render — no pool body wrapper.
    expect(root.innerHTML).not.toContain("lv-pool-body");
  });

  test("clicking a round head expands it and wraps ghosts in .lv-pool-body", () => {
    const store = buildLadder();
    let capturedListener: ((ev: MouseEvent) => void) | undefined;
    const root = fakeRoot();
    // Intercept the addEventListener call so we can fire click events.
    (root as unknown as { addEventListener(t: string, fn: (ev: MouseEvent) => void): void }).addEventListener = (
      _type: string,
      fn: (ev: MouseEvent) => void,
    ) => {
      capturedListener = fn;
    };
    const view = createLadderView(root, { store, registry: stressRegistry });
    view.refresh({ round: 1, runId: "run-0" });

    // Simulate a click on the round-1 head button.
    const fakeBtn = {
      closest(sel: string) {
        if (sel === "[data-lvround]") return { getAttribute: () => "1" };
        return null;
      },
    } as unknown as HTMLElement;
    capturedListener!({ target: fakeBtn } as unknown as MouseEvent);

    // After expansion, ghosts are wrapped in .lv-pool-body.
    expect(root.innerHTML).toContain("lv-pool-body");
    // And the ghost entries are inside it.
    const bodyStart = root.innerHTML.indexOf("lv-pool-body");
    const ghostStart = root.innerHTML.indexOf("lv-ghost");
    expect(ghostStart).toBeGreaterThan(bodyStart);
  });

  test("collapsing a round removes .lv-pool-body", () => {
    const store = buildLadder();
    let capturedListener: ((ev: MouseEvent) => void) | undefined;
    const root = fakeRoot();
    (root as unknown as { addEventListener(t: string, fn: (ev: MouseEvent) => void): void }).addEventListener = (
      _type: string,
      fn: (ev: MouseEvent) => void,
    ) => {
      capturedListener = fn;
    };
    const view = createLadderView(root, { store, registry: stressRegistry });
    view.refresh({ round: 1, runId: "run-0" });

    const fakeBtn = {
      closest(sel: string) {
        if (sel === "[data-lvround]") return { getAttribute: () => "1" };
        return null;
      },
    } as unknown as HTMLElement;
    // Expand then collapse.
    capturedListener!({ target: fakeBtn } as unknown as MouseEvent);
    expect(root.innerHTML).toContain("lv-pool-body");
    capturedListener!({ target: fakeBtn } as unknown as MouseEvent);
    expect(root.innerHTML).not.toContain("lv-pool-body");
  });
});

import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { resolve, dirname } from "path";

const __dirname = dirname(fileURLToPath(import.meta.url));

describe("run-screen notice/error stay in flow", () => {
  const src = readFileSync(resolve(__dirname, "run-screen.ts"), "utf8");

  test("run-screen.ts never sets els.notice.hidden", () => {
    // The JS source must not set .hidden on notice — the element reserves space
    // via CSS min-height and content is cleared instead of hiding.
    expect(src).not.toMatch(/els\.notice\.hidden\s*=/);
  });

  test("run-screen.ts never sets els.error.hidden", () => {
    expect(src).not.toMatch(/els\.error\.hidden\s*=/);
  });
});

// ---------------------------------------------------------------------------
// Slice-3 fix round pins (Cass refutations at 375×667). jsdom can't see wrap
// or collapse, so each fix is pinned two ways: the CSS rules that hold the
// geometry are parsed and checked arithmetically, and the shop reserve is
// driven through createRunScreen with a modeled phone layout (2 cards per
// row) — exactly the roll → buy collapse the verifier measured.
// ---------------------------------------------------------------------------

import { vi } from "vitest";
import { DEFAULT_RUN_POOL, stressRegistry as registry } from "../src/index.js";
import { createRunScreen, type RunScreenDeps } from "./run-screen.js";

const css = readFileSync(resolve(__dirname, "style.css"), "utf8");
const runScreenSrc = readFileSync(resolve(__dirname, "run-screen.ts"), "utf8");

/** The first `selector { … }` body in the stylesheet (selectors are unique). */
function ruleBody(selector: string | RegExp): string {
  const m =
    typeof selector === "string"
      ? css.match(new RegExp(selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + String.raw`\s*\{([^}]*)\}`))
      : css.match(selector);
  expect(m, `style.css should have a rule for ${selector}`).not.toBeNull();
  return m![1]!;
}

describe("notice/error strips are fixed-height at phone width (refutations 1–2)", () => {
  test("the phone strip rule fixes height at two lines and clamps overflow", () => {
    // A min-height let real strings wrap and GROW the strip (fight −87px on
    // every fuse, ladder +16px on every full-line error). A fixed height +
    // line clamp makes the strip's size independent of its content.
    const body = ruleBody(/\.run-notice,\s*#run-error\s*\{([^}]*)\}/);
    expect(body).toMatch(/height:\s*2\.4rem/);
    expect(body).not.toMatch(/min-height/);
    expect(body).toMatch(/-webkit-line-clamp:\s*2/);
    expect(body).toMatch(/overflow:\s*hidden/);
  });

  test("run-screen.ts mirrors the full string onto the title attribute", () => {
    // The clamp may ellipsize a >2-line string — the title must carry it all.
    expect(runScreenSrc).toMatch(/els\.notice\.title\s*=/);
    expect(runScreenSrc).toMatch(/els\.error\.title\s*=/);
  });
});

describe("hit extensions clip at gap midpoints (refutation 4 + low e)", () => {
  test("reorder arrows: inner edges split at the gap midpoint, earlier sibling carries the snap guard", () => {
    // Centered 44px boxes overlapped: ◂'s right half fired ▸ (the later
    // sibling wins hit-testing) and dead-tapped when ▸ was disabled. The fix:
    // ▸'s hit edge sits exactly at the midpoint (it wins where both paint);
    // ◂ carries a 2px guard past it, because Chromium pixel-snaps hit rects
    // and exactly-tiled halves left a 1px dead seam (e2e probe measured it).
    // Ownership lands within 1px of the midpoint, crack-free.
    const pair = ruleBody(".run-move button::after");
    expect(pair).not.toMatch(/translate\(-50%, -50%\)/);
    expect(pair).not.toMatch(/max\(100%, 44px\)/);
    const inset = pair.match(/inset:\s*(-?[\d.]+)rem\s+(-?[\d.]+)px\s+(-?[\d.]+)rem\s+(-?[\d.]+)px/);
    expect(inset, "arrow ::after should use explicit insets").not.toBeNull();
    const first = ruleBody(".run-move button:first-child::after");
    const last = ruleBody(".run-move button:last-child::after");
    const right = first.match(/right:\s*calc\((-?[\d.]+)rem - 2px\)/);
    const left = last.match(/left:\s*(-?[\d.]+)rem/);
    expect(right, "◂ extends right by half the gap + the 2px snap guard").not.toBeNull();
    expect(left, "▸ extends left by exactly half the gap").not.toBeNull();
    const gap = Number(ruleBody(".run-move").match(/gap:\s*([\d.]+)rem/)![1]);
    // Both rem parts reach exactly the midpoint; only ◂'s 2px guard crosses
    // it, and ▸ (the later sibling) deterministically wins that overlap.
    expect(Number(right![1]) * -2).toBeCloseTo(gap);
    expect(Number(left![1]) * -2).toBeCloseTo(gap);
    // A disabled arrow must never swallow the live sibling's seam tap.
    expect(ruleBody(".run-move button:disabled")).toMatch(/pointer-events:\s*none/);
    // Vertical: clipped above (low e — a tap above the arrows inspects, it
    // doesn't reorder), never extending past the card's own bottom padding.
    const top = Number(inset![1]);
    expect(top).toBeLessThan(0);
    expect(top).toBeGreaterThanOrEqual(-0.15);
  });

  test("buy button: wide is fine (no horizontal sibling) but vertical clips at the gap midpoint", () => {
    // Low e: the old centered box reached ~12px up into the card body — a tap
    // just above buy bought instead of inspecting.
    const body = ruleBody(".run-buy::after");
    expect(body).not.toMatch(/translate\(-50%, -50%\)/);
    expect(body).not.toMatch(/height:\s*max/);
    expect(Number(body.match(/top:\s*(-?[\d.]+)rem/)![1])).toBeGreaterThanOrEqual(-0.15);
    expect(Number(body.match(/bottom:\s*(-?[\d.]+)rem/)![1])).toBeGreaterThanOrEqual(-0.3);
  });

  test("status chips: each owns its visual plus its half-gap share (snap guard on the right)", () => {
    // The same overlap bug found on adjacent chips (exclusive width ≈30px).
    // The arrows' seam scheme sideways: the left hit edge sits exactly at the
    // gap midpoint (the later chip wins where both paint), the right edge
    // carries the 2px snap guard so a pixel-snapped seam can never go dead.
    // Vertical stays the bare half-gap — no contesting the buy row below.
    const body = ruleBody(".chip::after");
    expect(body).not.toMatch(/max\(100%, 44px\)/);
    const inset = body.match(/inset:\s*(-?[\d.]+)rem\s+calc\((-?[\d.]+)rem - 2px\)\s+(-?[\d.]+)rem\s+(-?[\d.]+)rem/);
    expect(inset, "chip ::after: vertical half-gap, right +2px guard, left exact half-gap").not.toBeNull();
    const gap = Number(ruleBody(".chips").match(/gap:\s*([\d.]+)rem/)![1]);
    expect(Number(inset![1])).toBeLessThan(0);
    expect(Number(inset![1]) * -2).toBeLessThanOrEqual(gap + 1e-9); // vertical: never overlaps
    expect(Number(inset![2]) * -2).toBeCloseTo(gap); // right rem part: the midpoint
    expect(Number(inset![4]) * -2).toBeCloseTo(gap); // left edge: exactly the midpoint
  });

  test("inspector close stays a real ≥44px target (low a)", () => {
    const phone = css.slice(css.indexOf("@media (max-width: 700px)"));
    const body = phone.match(/#ins-close\s*\{([^}]*)\}/)![1]!;
    expect(body).toMatch(/min-width:\s*44px/);
    expect(body).toMatch(/min-height:\s*44px/);
  });
});

describe("shop row reserves the rolled offer count's layout (refutation 3)", () => {
  // A fake DOM with a layout model: 2 offer cards per row (the measured 375px
  // wrap), 130px per row. The old one-row min-height fails this: buying from
  // a 3-offer two-row shop collapsed 234→131px and moved the fight button.
  interface FakeEl {
    hidden: boolean;
    textContent: string;
    title: string;
    innerHTML: string;
    value: string;
    disabled: boolean;
    style: Record<string, string>;
    offsetHeight: number;
    getBoundingClientRect(): { height: number };
    addEventListener(type: string, fn: (ev: unknown) => void): void;
    fire(type: string, ev: unknown): void;
    querySelector(sel: string): null;
    append(): void;
    classList: { toggle(): void };
    scrollIntoView(): void;
  }

  function makeEl(): FakeEl {
    const listeners: Record<string, ((ev: unknown) => void)[]> = {};
    return {
      hidden: false,
      textContent: "",
      title: "",
      innerHTML: "",
      value: "",
      disabled: false,
      style: {},
      offsetHeight: 0,
      // The code measures the fractional rect height; the model derives it
      // from the same offsetHeight the layout stub computes.
      getBoundingClientRect() {
        return { height: this.offsetHeight };
      },
      addEventListener(type, fn) {
        (listeners[type] ??= []).push(fn);
      },
      fire(type, ev) {
        for (const fn of listeners[type] ?? []) fn(ev);
      },
      querySelector: () => null,
      append() {},
      classList: { toggle() {} },
      scrollIntoView() {},
    };
  }

  const CARDS_PER_ROW = 2;
  const ROW_PX = 130;

  function harness() {
    vi.stubGlobal("window", { scrollTo() {}, addEventListener() {}, matchMedia: () => ({ matches: false }) });
    const names = [
      "newPanel", "newForm", "seed", "dice", "newError", "champ", "warn", "shopPanel", "head", "next",
      "notice", "shopRow", "rerollButton", "line", "fightButton", "stakes", "error", "battlePanel",
      "battleHead", "battleMount", "battleBar", "outcome", "continueButton", "skipButton", "endPanel",
      "endHead", "endStats", "endLine", "newRunButton", "ladderPanel", "ladderBody",
    ] as const;
    const els = Object.fromEntries(names.map((n) => [n, makeEl()])) as Record<(typeof names)[number], FakeEl>;
    // The layout model: rows of two cards, 130px per row, 18px placeholder.
    Object.defineProperty(els.shopRow, "offsetHeight", {
      get() {
        const cards = (els.shopRow.innerHTML.match(/data-offer=/g) ?? []).length;
        return cards === 0 ? 18 : Math.ceil(cards / CARDS_PER_ROW) * ROW_PX;
      },
    });
    const kv = new Map<string, string>();
    const deps = {
      storage: {
        getItem: (k: string) => kv.get(k) ?? null,
        setItem: (k: string, v: string) => void kv.set(k, v),
        removeItem: (k: string) => void kv.delete(k),
      },
      store: new InMemoryLadderStore(),
      pool: DEFAULT_RUN_POOL,
      registry,
      viewer: { load() {}, stop() {}, toEnd() {} },
      viewerHost: makeEl(),
      viewerHome: makeEl(),
    } as unknown as RunScreenDeps;
    createRunScreen(els as never, deps);
    els.seed.value = "7";
    els.newForm.fire("submit", { preventDefault() {} });
    return els;
  }

  const cardCount = (html: string): number => (html.match(/data-offer=/g) ?? []).length;

  test("a fresh roll reserves its own wrapped height, and a buy keeps it", () => {
    const els = harness();
    // Round 1 rolls 3 offers → two modeled rows reserved.
    expect(cardCount(els.shopRow.innerHTML)).toBe(3);
    expect(els.shopRow.style.minHeight).toBe(`${2 * ROW_PX}px`);
    // Buy: 2 offers remain (one row), but the reserve still holds the rolled
    // two-row layout — the fight button must not move on the loop's most
    // common action.
    els.shopRow.fire("click", {
      target: { closest: (sel: string) => (sel === "[data-buy]" ? { getAttribute: () => "0" } : null) },
    });
    expect(cardCount(els.shopRow.innerHTML)).toBe(2);
    expect(els.shopRow.style.minHeight).toBe(`${2 * ROW_PX}px`);
    vi.unstubAllGlobals();
  });

  test("the measurement leaves no filler cards behind", () => {
    const els = harness();
    els.shopRow.fire("click", {
      target: { closest: (sel: string) => (sel === "[data-buy]" ? { getAttribute: () => "0" } : null) },
    });
    // Real cards only: indices 0..1 — the measuring fillers are gone.
    expect(els.shopRow.innerHTML).not.toContain('data-offer="2"');
    vi.unstubAllGlobals();
  });
});
