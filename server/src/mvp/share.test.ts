// M4-7: share cards. Both render for a seeded store, in English and Russian;
// every text drawn fits the card; a long name is shrunk or cut, never spilled.
import { writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import type { Champion, LineUnit, PlayerRef } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { createMvpApp } from "./app.js";
import { creditUnit } from "./credits.js";
import { poolContent, seedUnits } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { mvpServerApp } from "./server.js";
import { championSvg, fit, SHARE_H, SHARE_W, shareLang, shareMeta, textWidth, unitSvg, wrap } from "./share.js";
import { MemoryMvpStore } from "./store.js";

const LONG = "Grandmaster of the Endless Lighthouse Choir";
const ann: PlayerRef = { id: "p-ann", name: "Annabelle_the_Unbeatable", bot: false };

function world() {
  const store = new MemoryMvpStore();
  seedUnits(store, new Date("2026-10-08T08:00:00.000Z"));
  const rt = mvpRuntime({ content: poolContent(store), store, dev: true });
  const [a, b, c, d, e] = rt.content.units;
  const fused: LineUnit = { ...lineUnitOf(d!, "u4", 1, rt.rules), kind: "fused", emoji: `${d!.emoji}${e!.emoji}`, name: LONG, fusion: { first: d!.id, second: e!.id, name: LONG, discoveredBy: ann } };
  const line = [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 1, rt.rules), lineUnitOf(c!, "u3", 2, rt.rules), fused, lineUnitOf(e!, "u5", 1, rt.rules)];
  const day = rt.today();
  const champ: Champion = { seq: day.seq, day: day.day, player: ann, line, since: "2026-10-08T01:00:00.000Z", contentVersion: rt.content.version };
  store.addPlayer(ann);
  store.putChampion(champ);
  return { rt, store, app: createMvpApp(rt), champ };
}

/** Every plain <text> in the SVG: its left and right edge, measured with the card's own fonts. */
function textBoxes(svg: string) {
  const re = /<text x="([\d.]+)" y="[\d.]+" font-family="([^"]+)" font-weight="(\d+)" font-size="([\d.]+)" fill="[^"]+" ([^>]*)>([^<]*)<\/text>/g;
  const out: { text: string; left: number; right: number }[] = [];
  for (const m of svg.matchAll(re)) {
    const text = m[6]!.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&amp;/g, "&");
    const w = textWidth(text, Number(m[4]), { family: m[2]!, weight: Number(m[3]) });
    const x = Number(m[1]);
    const left = m[5]!.includes('text-anchor="middle"') ? x - w / 2 : m[5]!.includes('text-anchor="end"') ? x - w : x;
    out.push({ text, left, right: left + w });
  }
  return out;
}
const expectFits = (svg: string) => {
  const boxes = textBoxes(svg);
  expect(boxes.length).toBeGreaterThan(5);
  for (const b of boxes) {
    expect(b.left, b.text).toBeGreaterThanOrEqual(30);
    expect(b.right, b.text).toBeLessThanOrEqual(SHARE_W - 30);
  }
};
const pngSize = (buf: ArrayBuffer) => {
  const v = new DataView(buf);
  expect([...new Uint8Array(buf, 0, 4)]).toEqual([0x89, 0x50, 0x4e, 0x47]);
  return { w: v.getUint32(16), h: v.getUint32(20) };
};

describe("share cards (M4-7)", () => {
  it("the champion of the day: name, five cards, slayers, the address; English and Russian", () => {
    const { rt, champ } = world();
    const en = championSvg(rt, champ.seq, "en")!;
    expect(en).toContain("Champion of Oct 8, 2026");
    expect(en).toContain("@Annabelle_the_Unbeatable");
    expect(en).toContain("0 slayers");
    expect(en).toContain("arena.makscee.ru");
    expect(en).toContain("AWOKEN"); // the 3-copy unit
    expect(en).toContain("FUSED");
    expect(en.match(/<polygon/g)!.length).toBeGreaterThanOrEqual(6); // the frame and five cards
    expectFits(en);
    const ru = championSvg(rt, champ.seq, "ru")!;
    expect(ru).toContain("Чемпион 8 октября 2026");
    expect(ru).toContain("ПРОБУЖДЁН");
    expectFits(ru);
  });

  it("a long unit name is shrunk, then cut with …, inside its card", () => {
    const body = { family: "Rajdhani, Play", weight: 600 };
    const f = fit(LONG, body, 176, 30, 18);
    expect(f.text.endsWith("…")).toBe(true);
    expect(textWidth(f.text, f.size, body)).toBeLessThanOrEqual(176);
    expect(fit("Hog", body, 176, 30, 18)).toEqual({ text: "Hog", size: 30 });
    const lines = wrap(`${LONG} `.repeat(8), body, 28, 500, 3);
    expect(lines).toHaveLength(3);
    for (const l of lines) expect(textWidth(l, 28, body)).toBeLessThanOrEqual(500);
    expect(lines[2]!.endsWith("…")).toBe(true);
  });

  it("one unit big: its rule, who it's by, NEW; Russian; a unit that left; none", () => {
    const { rt, store } = world();
    const u = rt.content.units.find((x) => x.tier === 2)!;
    store.addPlayer({ id: "p-bob", name: "bob", bot: false });
    expect(creditUnit(rt, u.id, "p-bob")).toBe(true);
    const en = unitSvg(rt, u.id, "en")!;
    expect(en).toContain(u.name.replace(/&/g, "&amp;"));
    expect(en).toContain("idea by @bob");
    expect(en).toContain(">NEW<");
    expect(en).toContain("SLEEPING");
    expect(en).toContain("AWOKEN");
    expectFits(en);
    const ru = unitSvg(rt, u.id, "ru")!;
    expect(ru).toContain("идея @bob");
    expect(ru).toContain("НОВЫЙ");
    expectFits(ru);
    const seed = unitSvg(rt, rt.content.units[0]!.id, "en")!;
    expect(seed).toContain("one of the first units");
    expect(seed).not.toContain(">NEW<");
    expect(unitSvg(rt, "no-such-unit", "en")).toBeNull();
  });

  it("GET /share/…png: 1200×630 PNGs, cached per content, 404 for nothing", async () => {
    const { app, champ, rt } = world();
    const get = (p: string, h: Record<string, string> = {}) => app.request(`/api/v1${p}`, { headers: h });
    const r = await get(`/share/champion/${champ.seq}.png`);
    expect(r.status).toBe(200);
    expect(r.headers.get("content-type")).toBe("image/png");
    const buf = await r.arrayBuffer();
    expect(pngSize(buf)).toEqual({ w: SHARE_W, h: SHARE_H });
    const etag = r.headers.get("etag")!;
    expect((await get(`/share/champion/${champ.seq}.png`, { "If-None-Match": etag })).status).toBe(304);
    const ru = await get(`/share/champion/${champ.seq}.png?lang=ru`);
    expect(ru.headers.get("etag")).not.toBe(etag);
    const unitId = rt.content.units.find((x) => x.forms.awoken.also?.length || x.forms.sleeping.does.length > 1)?.id ?? rt.content.units[3]!.id;
    expect(creditUnit(rt, unitId, ann.id)).toBe(true);
    const u = await get(`/share/unit/${unitId}.png`, { "Accept-Language": "ru-RU,ru;q=0.9" });
    expect(u.status).toBe(200);
    expect(pngSize(await u.arrayBuffer())).toEqual({ w: SHARE_W, h: SHARE_H });
    expect((await get(`/share/champion/${champ.seq + 1}.png`)).status).toBe(404);
    expect((await get(`/share/champion/abc.png`)).status).toBe(404);
    expect((await get(`/share/unit/nope.png`)).status).toBe(404);
    if (process.env.SHARE_OUT) {
      // SHARE_OUT=<dir> keeps the four PNGs, for a look (and the PR).
      for (const [name, path] of [["champion-en", `/share/champion/${champ.seq}.png`], ["champion-ru", `/share/champion/${champ.seq}.png?lang=ru`], ["unit-en", `/share/unit/${unitId}.png`], ["unit-ru", `/share/unit/${unitId}.png?lang=ru`]] as const)
        writeFileSync(`${process.env.SHARE_OUT}/${name}.png`, Buffer.from(await (await get(path)).arrayBuffer()));
    }
  });

  it("the language: ?lang wins, then Accept-Language, else English", () => {
    expect(shareLang("ru")).toBe("ru");
    expect(shareLang(undefined, "ru-RU,en;q=0.8")).toBe("ru");
    expect(shareLang("en", "ru")).toBe("en");
    expect(shareLang(undefined, "en-GB")).toBe("en");
    expect(shareLang("xx")).toBe("en");
  });

  it("a shared link's page carries Open Graph tags with the card", async () => {
    const { rt, champ } = world();
    const meta = shareMeta(rt, new URL("http://x/?share=champion"), "https://arena.example/arena")!;
    expect(meta).toContain(`<meta property="og:image" content="https://arena.example/arena/api/v1/share/champion/${champ.seq}.png" />`);
    expect(meta).toContain('og:title" content="Champion of Oct 8, 2026: @Annabelle_the_Unbeatable"');
    expect(meta).toContain('twitter:card" content="summary_large_image"');
    expect(shareMeta(rt, new URL("http://x/?share=champion&lang=ru"), "https://a/arena")).toContain("champion/1.png?lang=ru");
    expect(shareMeta(rt, new URL("http://x/?share=unit&unit=" + rt.content.units[0]!.id), "https://a/arena")).toContain(`/share/unit/${rt.content.units[0]!.id}.png`);
    expect(shareMeta(rt, new URL("http://x/"), "https://a")).toBeNull();
    expect(shareMeta(rt, new URL("http://x/?share=champion&day=99"), "https://a")).toBeNull();

    // Through the page server: the built index.html with the tags in its head.
    const dir = (await import("node:fs")).mkdtempSync(`${(await import("node:os")).tmpdir()}/share-`);
    writeFileSync(`${dir}/index.html`, "<html><head><title>Arena of Ideas</title></head><body></body></html>");
    const page = mvpServerApp(createMvpApp(rt), { staticRoot: dir, build: null, pageMeta: (u, l) => shareMeta(rt, u, "https://a/arena", l) });
    const html = await (await page.request("/?share=champion")).text();
    const head = html.slice(0, html.indexOf("</head>"));
    expect(head).toContain('og:image" content="https://a/arena/api/v1/share/champion/1.png" />');
    expect(await (await page.request("/")).text()).not.toContain("og:image");
  });
});
