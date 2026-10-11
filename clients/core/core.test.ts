// clients/core (#957): DOM-free and Node-free by its imports, and its client,
// bot and text renderers play a whole run against the real API.
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { MVP_RULES } from "../../src/mvp/contract.js";
import { createMvpApp } from "../../server/src/mvp/app.js";
import { mvpContent } from "../../server/src/mvp/content.js";
import { battle as enBattle } from "../../mobile/i18n/en/battle.js";
import { battle as ruBattle } from "../../mobile/i18n/ru/battle.js";
import { ApiError, ArenaClient, battleText, botDecision, cardText, glossaryText, runText, traceText, TURN_HEADS } from "./index.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

/** Every file clients/core reaches through its imports, from index.ts. */
function importGraph(): { files: string[]; bad: string[] } {
  const files = new Set<string>();
  const bad: string[] = [];
  const walk = (file: string) => {
    if (files.has(file)) return;
    files.add(file);
    const src = readFileSync(file, "utf8");
    for (const m of src.matchAll(/^\s*(?:import|export)\s[^;]*?from\s+"([^"]+)"/gm)) {
      const spec = m[1]!;
      const rel = file.slice(root.length + 1);
      if (!spec.startsWith(".")) {
        bad.push(`${rel} imports ${spec}`);
        continue;
      }
      const target = resolve(dirname(file), spec).replace(/\.js$/, ".ts");
      if (/\/(server|mobile|web|e2e|scripts)\//.test(target.slice(root.length))) bad.push(`${rel} imports ${spec}`);
      walk(target.endsWith(".json") ? target.replace(/\.ts$/, "") : target);
    }
  };
  walk(resolve(root, "clients/core/index.ts"));
  return { files: [...files].filter((f) => !f.endsWith(".test.ts")), bad };
}

describe("clients/core", () => {
  it("imports nothing from Node, the DOM, npm, the server or the app", () => {
    const { files, bad } = importGraph();
    expect(files.length).toBeGreaterThan(5);
    expect(bad).toEqual([]);
    // Nor uses their globals (comments and strings aside).
    const used = files.flatMap((f) => {
      if (f.endsWith(".json")) return [];
      const code = readFileSync(f, "utf8")
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/\/\/.*$/gm, "")
        .replace(/"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'|`(?:[^`\\]|\\.)*`/g, '""');
      return [...code.matchAll(/\b(document|window|localStorage|sessionStorage|navigator|HTMLElement|process|Buffer|require|__dirname)\b(?!\s*:)/g)].map((m) => `${f.slice(root.length + 1)}: ${m[1]}`);
    });
    expect(used).toEqual([]);
  });

  it("plays a whole run against the API: client, bot and text", async () => {
    const app = createMvpApp({ content: mvpContent(), dev: true });
    const client = new ArenaClient({ api: "http://arena.test/api/v1", fetch: (url, init) => Promise.resolve(app.request(String(url), init)) });
    await expect(client.home()).resolves.toMatchObject({ rules: expect.any(Object) });
    const content = await client.content();
    await client.register("Core");
    let run = await client.startRun();
    expect(runText(run, content, MVP_RULES)).toMatch(/^Arena · Round 1\/\d+ · ♥\d+ · \d+g/);
    expect(runText(run, content, MVP_RULES)).toContain("SHOP\n  [0] ");
    let rerolled = 0;
    let battleId = "";
    for (let i = 0; i < 400 && run.phase !== "over"; i++) {
      const d = botDecision(run, content, MVP_RULES, rerolled);
      rerolled = d.kind === "reroll" ? rerolled + 1 : d.kind === "fight" ? 0 : rerolled;
      const r = await client.decide(run.runId, d);
      run = r.run;
      if (r.fight) battleId = r.fight.battleId;
    }
    expect(run.phase).toBe("over");
    expect(runText(run, content, MVP_RULES)).toContain("LAST   ");
    const b = await client.battle(battleId);
    const steps = battleText(b, { you: "A" });
    expect(steps).toMatch(/^Battle .+\nA: .+\nB: .+\n/);
    expect(steps).toMatch(/\nTurn 1\n/);
    expect(steps).toMatch(/(You win|They win|draw)/i);
    const id = Number(/#(\d+)/.exec(steps.split("Turn 1")[1]!)![1]);
    expect(traceText(b, id).split("\n").length).toBeGreaterThan(0);
    expect(battleText(b, { you: "A", view: "summary" }).split("\n").length).toBe(5);
    const ru = battleText(b, { you: "A", lang: "ru" });
    expect(ru).toMatch(/[а-я]/);
    // #960: the turn headers are Russian too, never "Start", "Turn N" or "Turns a–b".
    expect(ru).toMatch(/^Старт$/m);
    expect(ru).toMatch(/^Ход 1$/m);
    expect(ru).not.toMatch(/^(Start|Turns? \d)/m);
    // English again after a Russian replay.
    expect(battleText(b, { you: "A" })).toBe(steps);
    await expect(client.run("nope")).rejects.toBeInstanceOf(ApiError);
  });

  it("battle turn headers are the phone's words (#960)", () => {
    expect(TURN_HEADS.en.start).toBe(enBattle["battle.timeline.start"]);
    expect(TURN_HEADS.en.turn).toBe(enBattle["battle.turn"]);
    expect(TURN_HEADS.ru.start).toBe(ruBattle["battle.timeline.start"]);
    expect(TURN_HEADS.ru.turn).toBe(ruBattle["battle.turn"]);
  });

  it("card and glossary text, English and Russian", () => {
    const content = mvpContent();
    const u = content.units[0]!;
    expect(cardText(u, content)).toMatch(new RegExp(`^${u.emoji} ${u.name} · T${u.tier} · ${u.base.pwr}/${u.base.hp}\\n.+\\nSleeping: .+\\nAwoken: .+$`));
    expect(cardText(u, content, { lang: "ru" })).toContain("Пробуждён: ");
    expect(glossaryText(content)).toMatch(/^PWR: Power/);
    expect(glossaryText(content, { lang: "ru" })).toMatch(/[а-я]/);
  });
});
