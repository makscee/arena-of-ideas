// Title hub (B·Arena slice B) — the title screen is the always-on 3-column HUB:
// the ideas creation ladder (left), the wordmark + run actions (center), and
// the Arena Tower (right). The live behaviour is wired in main.ts (two reused
// slice-C renders dropped into the hub columns); this pins the static shell so
// the mount points, the wordmark, the New Run / Continue split, and every
// preserved player/utility entry never silently drop. Read as text — the suite
// runs in node with no DOM, the way the other render tests do.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const html = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "index.html"), "utf8");
const start = html.indexOf('id="title-view"');
const titleView = html.slice(start, html.indexOf("</section>", start));

describe("title hub shell (index.html)", () => {
  test("the title view is a three-column hub grid", () => {
    expect(titleView).toContain('class="hub-grid"');
  });

  test("the center carries the Chakra-Petch wordmark", () => {
    expect(titleView).toContain('class="title-name'); // the font probe pins this class
    expect(titleView).toContain("Arena");
    expect(titleView).toContain("of Ideas");
  });

  test("the New Run primary keeps the #title-play nav id; Continue is present but hidden", () => {
    expect(titleView).toContain('id="title-play"');
    expect(titleView).toMatch(/New Run/);
    expect(titleView).toMatch(/id="title-continue"[^>]*hidden/);
  });

  test("the left column mounts the ideas ladder with its submit CTA; the right mounts the tower", () => {
    expect(titleView).toContain('id="hub-ideas-list"');
    expect(titleView).toContain('id="hub-ideas-reveal"'); // the magenta "submit an idea" footer
    expect(titleView).toContain('id="hub-tower-body"');
  });

  test("the column headers stay reachable as the full-screen routes", () => {
    expect(titleView).toContain('id="title-ideas"'); // ideas header → full ideas view
    expect(titleView).toContain('id="title-leaderboard"'); // tower header → full leaderboard view
    expect(titleView).toContain('id="title-codex"'); // codex action
  });

  test("the utility row keeps login / settings / history / dev and the login state reachable", () => {
    for (const id of [
      "title-login",
      "title-logout",
      "title-settings",
      "title-history",
      "title-dev",
      "title-id",
      "title-name",
      "title-net-warn",
    ]) {
      expect(titleView).toContain(`id="${id}"`);
    }
  });
});
