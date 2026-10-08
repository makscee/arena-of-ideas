import { describe, expect, it } from "vitest";
import { MVP_RULES, type PlayerRef } from "../../../src/mvp/contract.js";
import { mvpContent } from "./content.js";
import { MVP_JOBS, startMvpJobs } from "./jobs.js";
import { startRun } from "./runs.js";
import { mvpRuntime, type MvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };

// 30 s each: building the content and starting the jobs is slow under load (R3-26).
describe("MVP runtime: one world for routes, bots and jobs", { timeout: 30_000 }, () => {
  it("fills every default in one place and keeps the caller's hooks last", () => {
    const store = new MemoryMvpStore();
    const mine = {};
    const rt = mvpRuntime({ content: mvpContent(), store, hooks: [mine] });
    expect(rt.store).toBe(store);
    expect(rt.rules).toEqual(MVP_RULES);
    expect(rt.dev).toBe(false);
    expect(rt.hooks.at(-1)).toBe(mine);
    const [a, b] = rt.content.units;
    expect(rt.nameFusion(a!, b!, maks)).toEqual(rt.peekFusionName(a!, b!, maks));
  });

  it("starts runs on today's day, which the first call creates", () => {
    const at = new Date("2026-10-05T10:00:00.000Z");
    const rt = mvpRuntime({ content: mvpContent(), now: () => at });
    expect(rt.store.currentDay()).toBeUndefined();
    const run = startRun(rt, maks);
    expect(run.day).toBe(1);
    expect(rt.store.currentDay()).toMatchObject({ seq: 1, day: "2026-10-05", startedAt: at.toISOString() });
  });

  it("starts every job on the runtime and stops them all", () => {
    const rt = mvpRuntime({ content: mvpContent() });
    const log: string[] = [];
    const job = (name: string) => (r: MvpRuntime) => {
      log.push(`start ${name} ${r === rt}`);
      return () => log.push(`stop ${name}`);
    };
    const stop = startMvpJobs(rt, [job("a"), job("b")]);
    stop();
    expect(log).toEqual(["start a true", "start b true", "stop a", "stop b"]);
    expect(MVP_JOBS).toHaveLength(3);
    startMvpJobs(rt)();
  });
});
