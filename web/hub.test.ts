// Title hub (B·Arena slice B) — the title screen is the always-on 3-column HUB:
// the ideas creation ladder (left), the wordmark + run actions (center), and
// the Arena Tower (right). The live behaviour is wired in main.ts (two reused
// slice-C renders dropped into the hub columns); this pins the static shell and
// the PRD #112 slice-1 hierarchy: one primary run action, one idea route, one
// tower route, and secondary utilities in the strip. Read as text — the suite
// runs in node with no DOM, the way the other render tests do.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { TITLE_HUB_HIERARCHY } from "./title-screen.js";

const html = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "index.html"), "utf8");
const start = html.indexOf('id="title-view"');
const titleView = html.slice(start, html.indexOf("</section>", start));
const utilStart = titleView.indexOf('class="hub-util"');
const utilityStrip = titleView.slice(utilStart, titleView.indexOf("</div>", utilStart));
const actionsStart = titleView.indexOf('class="hub-actions"');
const hubActions = titleView.slice(actionsStart, titleView.indexOf("</div>", actionsStart));

function occurrences(haystack: string, needle: string): number {
  return haystack.split(needle).length - 1;
}

describe("title hub shell (index.html)", () => {
  test("the title view is a three-column hub grid", () => {
    expect(titleView).toContain('class="hub-grid"');
  });

  test("the center carries the Chakra-Petch wordmark", () => {
    expect(titleView).toContain('class="title-name'); // the font probe pins this class
    expect(titleView).toContain("Arena");
    expect(titleView).toContain("of Ideas");
  });

  test("the hierarchy defines one primary run action, one idea route, and one tower route", () => {
    expect(TITLE_HUB_HIERARCHY.primaryRunAction.id).toBe("title-play");
    expect(TITLE_HUB_HIERARCHY.primaryRunAction.label).toBe("▸ New Run");
    expect(TITLE_HUB_HIERARCHY.ideaRoutes).toEqual([{ id: "title-ideas", label: "Ideas" }]);
    expect(TITLE_HUB_HIERARCHY.towerRoutes).toEqual([{ id: "title-leaderboard", label: "Arena Tower" }]);
  });

  test("the New Run primary keeps the #title-play nav id; Continue is present but hidden", () => {
    expect(titleView).toContain(`id="${TITLE_HUB_HIERARCHY.primaryRunAction.id}"`);
    expect(titleView).toMatch(/New Run/);
    expect(titleView).toMatch(new RegExp(`id="${TITLE_HUB_HIERARCHY.continueRunAction.id}"[^>]*hidden`));
  });

  test("the center action row is reserved for run/continue plus the existing idea CTA", () => {
    expect(hubActions).toContain(`id="${TITLE_HUB_HIERARCHY.primaryRunAction.id}"`);
    expect(hubActions).toContain(`id="${TITLE_HUB_HIERARCHY.continueRunAction.id}"`);
    expect(hubActions).toContain('id="title-create-idea"'); // duplicate CTA remains for slice 2 to remove
    expect(hubActions).not.toContain('id="title-codex"');
  });

  test("the left column mounts the ideas ladder with its submit CTA; the right mounts the tower", () => {
    expect(titleView).toContain('id="hub-ideas-list"');
    expect(titleView).toContain('id="hub-ideas-reveal"'); // the magenta "submit an idea" footer
    expect(titleView).toContain('id="hub-tower-body"');
  });

  test("the one idea route and one tower route stay reachable as full-screen routes", () => {
    for (const route of [...TITLE_HUB_HIERARCHY.ideaRoutes, ...TITLE_HUB_HIERARCHY.towerRoutes]) {
      expect(titleView).toContain(`id="${route.id}"`);
      expect(occurrences(titleView, `id="${route.id}"`)).toBe(1);
    }
  });

  test("the secondary utility strip keeps codex / login / settings / history / dev reachable", () => {
    for (const id of TITLE_HUB_HIERARCHY.secondaryUtilityIds) {
      expect(utilityStrip).toContain(`id="${id}"`);
    }
    for (const id of ["title-id", "title-name", "title-net-warn"]) {
      expect(titleView).toContain(`id="${id}"`);
    }
  });
});
