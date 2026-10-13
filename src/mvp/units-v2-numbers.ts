// PWR and HP of the rules v2 pool, written by `npm run mvp:tune-v2`
// (scripts/tune-v2.ts): the numbers come from the tune, never from the
// designer. A unit missing here keeps its row's placeholder numbers.
// awoken: PWR and HP the unit gains when it awakens (the awoken knob).
export const V2_NUMBERS: Record<string, { pwr: number; hp: number; awoken?: { pwr: number; hp: number } }> = {
  "Hourglass": { pwr: 1, hp: 3, awoken: { pwr: 1, hp: 8 } },
  "Mirror": { pwr: 3, hp: 16, awoken: { pwr: 0, hp: 2 } },
  "Candle": { pwr: 6, hp: 16, awoken: { pwr: 2, hp: 8 } },
  "Gambler": { pwr: 1, hp: 3, awoken: { pwr: 2, hp: 8 } },
  "Nautilus": { pwr: 1, hp: 3, awoken: { pwr: 0, hp: 4 } },
  "Magnet": { pwr: 3, hp: 16, awoken: { pwr: 1, hp: 8 } },
  "Raven": { pwr: 1, hp: 8, awoken: { pwr: 0, hp: 6 } },
  "Seed": { pwr: 0, hp: 8, awoken: { pwr: 0, hp: 8 } },
  "Sniper": { pwr: 4, hp: 16, awoken: { pwr: 0, hp: 4 } },
  "Medic": { pwr: 1, hp: 16, awoken: { pwr: 1, hp: 8 } },
  "Leech": { pwr: 6, hp: 16, awoken: { pwr: 0, hp: 4 } },
  "Metronome": { pwr: 1, hp: 3, awoken: { pwr: 2, hp: 8 } },
  "Lone Wolf": { pwr: 3, hp: 6, awoken: { pwr: 0, hp: 8 } },
  "Drummer": { pwr: 1, hp: 3, awoken: { pwr: 0, hp: 4 } },
  "Bodyguard": { pwr: 1, hp: 10, awoken: { pwr: 2, hp: 8 } },
  "Chrysalis": { pwr: 0, hp: 3, awoken: { pwr: 0, hp: 6 } },
  "Trickster": { pwr: 2, hp: 8, awoken: { pwr: 1, hp: 8 } },
  "Understudy": { pwr: 1, hp: 13, awoken: { pwr: 2, hp: 8 } },
  "Gravedigger": { pwr: 1, hp: 3, awoken: { pwr: 0, hp: 2 } },
  "Conductor": { pwr: 1, hp: 3, awoken: { pwr: 0, hp: 4 } },
};
