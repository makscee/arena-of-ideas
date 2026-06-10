// Public API of the v5 kernel — everything a CLI or browser client imports.

export const KERNEL_VERSION = "5.0.0-alpha.0";

export { battle, toJSONL, winnerOf, TEAM_SIZE, FATIGUE_START, FATIGUE_RAMP, TURN_CAP } from "./battle.js";
export { renderReplay } from "./replay.js";
export type * from "./types.js";
export * from "./content/stress.js";
