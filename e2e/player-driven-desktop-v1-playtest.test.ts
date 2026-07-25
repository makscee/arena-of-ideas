import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import {
  DEFAULT_RUN_POOL,
  InMemoryLadderStore,
  battle,
  buy,
  challengeBoss,
  fuse,
  initRun,
  ladderFight,
  mergePool,
  reroll,
  stressAbilities,
  stressRegistry,
  synthClimbTeam,
  toBattleTeam,
} from "../src/index.js";
import { approveInto } from "../src/create/approve.js";
import { parseCandidateRecord } from "../src/create/candidates.js";
import { rngStep } from "../src/rng.js";
import { defaultApprovedRegistry } from "../server/src/content.js";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..");
const readJson = (path: string) => JSON.parse(readFileSync(path, "utf8")) as unknown;
const offerNames = (state: ReturnType<typeof initRun>) => state.offers.map((unit) => unit.name);

function seasonTwoContent() {
  const candidate = parseCandidateRecord(
    readJson(join(root, "candidates/frostbite-striker.json")),
    stressRegistry,
    stressAbilities,
    "candidates/frostbite-striker.json",
  );
  candidate.provenance = { ...candidate.provenance, creator: "Maks" };
  const approved = approveInto(
    defaultApprovedRegistry(),
    candidate,
    DEFAULT_RUN_POOL.map((unit) => unit.name),
    stressRegistry,
    stressAbilities,
  );
  return {
    pool: mergePool(DEFAULT_RUN_POOL, approved.units),
    statuses: stressRegistry,
    abilities: { ...stressAbilities, ...(approved.abilities ?? {}) },
  };
}

describe("player-driven-desktop-v1-playtest fixture integrity", () => {
  test("seed 0 reaches the released shop/fusion/tower plan only through production transitions", () => {
    const content = seasonTwoContent();
    const ladder = new InMemoryLadderStore();
    let state = initRun({ seed: 0, runId: "fixture-random-real-run-id", ...content });

    expect(offerNames(state)).toEqual(["Silencer", "Venomancer", "Silencer"]);
    state = buy(state, 1);
    state = ladderFight(state, ladder);
    expect(offerNames(state)).toEqual(["Brawler", "Bulwark", "Bulwark"]);

    state = ladderFight(state, ladder);
    expect(offerNames(state)).toEqual(["Venomancer", "Glacier", "Necromancer"]);
    state = buy(state, 0);
    state = reroll(state);
    expect(offerNames(state)).toEqual(["Squire", "Bulwark", "Frostbiter"]);
    state = buy(state, 2);
    state = ladderFight(state, ladder);

    expect(offerNames(state)).toEqual(["Frostbiter", "Silencer", "Venomancer", "Frostbiter"]);
    state = buy(state, 0);
    state = buy(state, 1);
    state = buy(state, 1);
    expect(state.team.map((unit) => [unit.name, unit.progression, unit.copies])).toEqual([
      ["Venomancer", "Awakened", 3],
      ["Frostbiter", "Awakened", 3],
    ]);

    state = fuse(state, 1, 0);
    const fused = state.team[0]!;
    expect({ name: fused.name, pwr: fused.base.pwr, hp: fused.base.hp, gold: state.gold, lives: state.lives }).toEqual({
      name: "Frostbiter + Venomancer",
      pwr: 7,
      hp: 25,
      gold: 3,
      lives: 4,
    });
    expect(fused.fusion?.parents.map((parent) => [parent.name, parent.def._creator ?? null])).toEqual([
      ["Frostbiter", "Maks"],
      ["Venomancer", null],
    ]);
    expect(fused.def.triggers).toEqual(fused.fusion?.parents[0]!.def.triggers);
    expect(fused.def.selectors).toEqual(fused.fusion?.parents[1]!.def.selectors);
    expect(fused.def.abilities).toEqual(["Frostbite", "Venom"]);

    // Reconstruct the exact real viewer input at this production boundary. The
    // empty ladder means the released synthClimbTeam fallback supplies the foe.
    const opponent = synthClimbTeam(state.round, state.rng, state.pool);
    const battleSeedDraw = rngStep(opponent.rng);
    const battleSeed = Math.floor(battleSeedDraw.value * 4294967296);
    const log = battle({
      teamA: toBattleTeam(state.team),
      teamB: opponent.team,
      seed: battleSeed,
      statuses: state.statuses,
      abilities: state.abilities,
    });
    const summon = log.find((event) => {
      if (event.type !== "Summon" || event.causedBy === null) return false;
      const death = log[event.causedBy];
      const hurt = death?.causedBy === null ? undefined : log[death?.causedBy ?? -1];
      const strike = hurt?.causedBy === null ? undefined : log[hurt?.causedBy ?? -1];
      return death?.type === "Death" && hurt?.type === "Hurt" && strike?.type === "Strike" && strike.striker.includes("Frostbiter + Venomancer");
    });
    const death = summon?.causedBy === null ? undefined : log[summon?.causedBy ?? -1];
    const hurt = death?.causedBy === null ? undefined : log[death?.causedBy ?? -1];
    const strike = hurt?.causedBy === null ? undefined : log[hurt?.causedBy ?? -1];
    expect([strike?.id, hurt?.id, death?.id, summon?.id]).toEqual([12, 13, 14, 15]);

    state = ladderFight(state, ladder);
    const finalFight = state.log.filter((event) => event.type === "FightFought").at(-1)!;
    expect(finalFight.battleSeed).toBe(battleSeed);
    expect(state.round).toBe(5);
    expect(ladder.champion()).toBeNull();

    state = challengeBoss(state, ladder);
    expect({ status: state.status, endedBy: state.endedBy, round: state.round }).toEqual({ status: "over", endedBy: "crown", round: 5 });
    expect(state.log.some((event) => event.type === "Founded" && event.floor === 1)).toBe(true);
    expect(ladder.bossAt(1)?.runId).toBe("fixture-random-real-run-id");
  }, 30_000);

  test("the browser tracer contains no forbidden state/rule injection and uses released boundaries", () => {
    const source = readFileSync(join(here, "player-driven-desktop-v1-playtest.mjs"), "utf8");
    for (const forbidden of [
      /localStorage\.setItem/,
      /addInitScript/,
      /document\.body\.innerHTML/,
      /\.dispatchEvent\(/,
      /data-dev-|devGold|devSpawn|devMutate/,
      /openRun\(/,
      /battleFixture=/,
      /eventLog|actorState|targetState/,
      /\b(?:INSERT|UPDATE|DELETE|REPLACE)\b/,
    ]) expect(source).not.toMatch(forbidden);
    expect(source).toContain('execFileSync("npm", ["run", "govern", "--", ...args]');
    for (const command of ["fixture-reset", "fixture-votes", "freeze", "ship", "bounce", "roll"])
      expect(source).toContain(`govern("${command}"`);
    expect(source).toContain('loginViaUi(maks, "maks@aoi62.test", "Maks")');
    expect(source).toContain('playerVoteRow.locator(".ideas-vote-down").click()');
    expect(source).toContain("maks.click('[data-fusion-first=\"0\"]')");
    expect(source).toContain('maks.click("[data-fusion-cancel]")');
    expect(source).toContain("maks.click('[data-fusion-first=\"1\"]')");
    expect(source).toContain("maks.click('[data-fuse=\"1:0\"]')");
    expect(source).toContain('page.locator("#scrub").fill');
    expect(source).toContain('localStorage.getItem("remote:aoi.run.v1")');
    expect(source).toContain('new Database(dbPath, { readonly: true, fileMustExist: true })');
    expect(source).toContain("page.screenshot({ path, fullPage: false })");
    expect(source).toContain("png.readUInt32BE(16)");
    expect(source).toContain("png.readUInt32BE(20)");
    expect(source).toContain("dimensions.width === PLAYTEST_DESKTOP.width && dimensions.height === PLAYTEST_DESKTOP.height");
  });

  test("AOI-63 battle golden remains byte-identical", () => {
    const bytes = readFileSync(join(root, "src/__fixtures__/golden-battles.jsonl"));
    expect(createHash("sha256").update(bytes).digest("hex")).toBe("5a6b2de2b96ceb51ee32f6c923c349a379bb6606a8960f785dfe602b7a59b7f8");
  });
});
