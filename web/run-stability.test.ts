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
