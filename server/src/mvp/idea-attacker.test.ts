// "The attacker" in the grammar and the idea reader (mission 3, M3-1,
// makscee/void-board#801): only a When on a hit has an attacker; the reader
// knows the word, and a near miss ("random" for "the weakest enemy") goes to
// the new-words log beside what it can't express.
import { describe, expect, it } from "vitest";
import { MVP_RULES, type IdeaArchetype } from "../../../src/mvp/contract.js";
import { formProblems } from "../../../src/mvp/forms.js";
import { formText } from "../../../src/mvp/form-text.js";
import { mvpPool, ROWS, type Row } from "../../../src/mvp/units.js";
import { checkReading, takenShapes } from "./idea-checks.js";
import { fakeIdeaReader, readerSystemPrompt, type IdeaReader } from "./idea-reader.js";
import { pickArchetype, readIdeas } from "./idea-reading.js";
import { grantIdea, writeIdea } from "./ideas.js";
import { seedUnits } from "./pool.js";
import { MemoryMvpStore } from "./store.js";

const HOG: IdeaArchetype = { name: "Quillback", emoji: "🦔", line: "Spines that answer every hit it takes." };
const row = (when: Row["when"], who: Row["who"], does = "Poison 1"): Row =>
  ({ name: "Quillback", emoji: "🦔", tier: 1, pwr: 1, hp: 5, when, who, does, archetype: HOG.line, awoken: { add: ["Hit 1"] } });

function world() {
  const store = new MemoryMvpStore();
  const deps = { store, rules: MVP_RULES, now: () => new Date("2026-10-08T08:00:00.000Z") };
  seedUnits(store, deps.now());
  const write = (text: string) => {
    grantIdea(deps, "p1");
    return writeIdea(deps, "p1", text);
  };
  return { store, deps, write };
}

describe("the attacker in the grammar", () => {
  it("reads as \"the attacker\" on the card", () => {
    const pool = mvpPool([row("hurt", "attacker")]);
    expect(formText(pool.units[0]!.forms.sleeping, pool.abilities)).toBe("Hit: 1 Poison to the attacker.");
    expect(formProblems(pool.units[0]!, pool)).toEqual([]);
  });

  it("is valid with hurt and allyHurt, refused with strike and the rest", () => {
    for (const when of ["hurt", "allyHurt"] as const) expect(formProblems(mvpPool([row(when, "attacker")]).units[0]!, mvpPool([row(when, "attacker")]))).toEqual([]);
    for (const when of ["strike", "start", "die", "allyDies"] as const) {
      const pool = mvpPool([row(when, "attacker")]);
      expect(formProblems(pool.units[0]!, pool)).toEqual([
        'quillback.sleeping: "the attacker" needs a When on a hit (hurt or allyHurt)',
        'quillback.awoken: "the attacker" needs a When on a hit (hurt or allyHurt)',
      ]);
    }
  });

  it("an awoken \"and\" clause on the attacker is checked too", () => {
    const pool = mvpPool([{ ...row("strike", "front"), awoken: { also: { who: "attacker", does: "Hit 1" } } }]);
    expect(formProblems(pool.units[0]!, pool)).toEqual(['quillback.awoken: "the attacker" needs a When on a hit (hurt or allyHurt)']);
  });

  it("a reading with attacker and strike fails the idea checks", () => {
    const r = checkReading({ when: "strike", who: "attacker", does: "Poison 1", awoken: { add: ["Hit 1"] } }, HOG, takenShapes(ROWS));
    expect("problems" in r && r.problems.join(" ")).toMatch(/the attacker" needs a When on a hit/);
    const ok = checkReading({ when: "hurt", who: "attacker", does: "Poison 1", awoken: { add: ["Hit 1"] } }, HOG, takenShapes(ROWS));
    expect("reading" in ok && ok.reading.text.sleeping).toBe("Hit: 1 Poison to the attacker.");
  });

  it("no live unit uses it", () => {
    expect(ROWS.filter((r) => r.who === "attacker" || r.awoken.who === "attacker" || r.awoken.also?.who === "attacker")).toEqual([]);
  });
});

describe("the attacker in the reader", () => {
  it("the prompt's word list has it", () => {
    expect(readerSystemPrompt([])).toMatch(/^ {2}attacker: whoever dealt the hit/m);
    expect(readerSystemPrompt([])).toMatch(/nearMiss/);
  });

  it("the fake reads \"poisons whoever hits it\" as Hit → the attacker → Poison", async () => {
    const { store, deps, write } = world();
    const idea = write("a hedgehog that poisons whoever hits it");
    await readIdeas(deps, fakeIdeaReader());
    pickArchetype(deps, "p1", idea.ideaId, 0);
    await readIdeas(deps, fakeIdeaReader());
    const r = store.idea(idea.ideaId)!;
    expect(r.state).toBe("pick-reading");
    expect(r.data.readings![0]!.text.sleeping).toBe("Hit: 1 Poison to the attacker.");
    expect(r.data.readings![0]).toMatchObject({ when: "hurt", who: "attacker", does: "Poison 1" });
    expect(store.wordRequests()).toEqual([]);
  });

  it("a near miss lands in the new-words log, and isn't shown as a missing word", async () => {
    const { store, deps, write } = world();
    const idea = write("a sniper that shoots the weakest enemy");
    await readIdeas(deps, fakeIdeaReader());
    pickArchetype(deps, "p1", idea.ideaId, 0);
    await readIdeas(deps, fakeIdeaReader());
    const r = store.idea(idea.ideaId)!;
    expect(r.state).toBe("pick-reading");
    expect(r.data.cantExpress).toBeUndefined();
    expect(store.wordRequests()).toEqual([
      { ideaId: idea.ideaId, word: "the weakest enemy (read as random)", part: "a sniper that shoots the weakest enemy", createdAt: "2026-10-08T08:00:00.000Z" },
    ]);
  });

  it("a scripted reader's near miss and can't-express both log, once each", async () => {
    const { store, deps, write } = world();
    const idea = write("a hedgehog that steals gold from front enemy who hits it");
    const fake = fakeIdeaReader();
    const reader: IdeaReader = {
      kind: "scripted",
      archetypes: fake.archetypes,
      faithful: fake.faithful,
      async readings(text, a, opts) {
        const out = await fake.readings(text, a, opts);
        return { ...out, nearMiss: [{ part: "front enemy who hits it", meant: "the attacker", used: "front" }, { part: "front enemy who hits it", meant: "the attacker", used: "front" }] };
      },
    };
    await readIdeas(deps, reader);
    pickArchetype(deps, "p1", idea.ideaId, 0);
    await readIdeas(deps, reader);
    expect(store.wordRequests().map((w) => w.word)).toEqual(["steal gold", "the attacker (read as front)"]);
    expect(store.idea(idea.ideaId)!.data.cantExpress).toEqual(["a hedgehog that steals gold from front enemy who hits it"]);
  });
});
