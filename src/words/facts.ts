// How the battle facts of the count amount read (§3.3).

import type { BattleFact } from "../types.js";

export const FACT_EN: Record<BattleFact, string> = {
  hitsTaken: "the hits it has taken",
  hitsTakenByAllies: "the hits its allies have taken",
  deadAllies: "its fallen allies",
  alliesAlive: "its living allies",
  enemiesAlive: "the living enemies",
  turn: "the turn number",
};

export const FACT_RU: Record<BattleFact, string> = {
  hitsTaken: "полученным ударам",
  hitsTakenByAllies: "ударам по союзникам",
  deadAllies: "павшим союзникам",
  alliesAlive: "живым союзникам",
  enemiesAlive: "живым врагам",
  turn: "номеру хода",
};
