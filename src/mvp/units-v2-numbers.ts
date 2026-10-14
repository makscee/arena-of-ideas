// PWR and HP of the rules v2 pool, written by `npm run mvp:tune-v2`
// (scripts/tune-v2.ts): the numbers come from the tune, never from the
// designer. A unit missing here keeps its row's placeholder numbers.
// awoken: PWR and HP the unit gains when it awakens (the awoken knob).
export const V2_NUMBERS: Record<string, { pwr: number; hp: number; awoken?: { pwr: number; hp: number } }> = {
  "Hourglass": { pwr: 1, hp: 4, awoken: { pwr: 3, hp: 12 } },
  "Mirror": { pwr: 4, hp: 19, awoken: { pwr: 0, hp: 2 } },
  "Candle": { pwr: 6, hp: 15, awoken: { pwr: 2, hp: 10 } },
  "Gambler": { pwr: 2, hp: 6, awoken: { pwr: 2, hp: 12 } },
  "Nautilus": { pwr: 1, hp: 3, awoken: { pwr: 0, hp: 4 } },
  "Magnet": { pwr: 7, hp: 24, awoken: { pwr: 2, hp: 8 } },
  "Raven": { pwr: 2, hp: 11, awoken: { pwr: 0, hp: 8 } },
  "Seed": { pwr: 0, hp: 11, awoken: { pwr: 0, hp: 12 } },
  "Sniper": { pwr: 4, hp: 14, awoken: { pwr: 0, hp: 4 } },
  "Medic": { pwr: 1, hp: 19, awoken: { pwr: 1, hp: 10 } },
  "Leech": { pwr: 6, hp: 16, awoken: { pwr: 0, hp: 6 } },
  "Metronome": { pwr: 2, hp: 6, awoken: { pwr: 2, hp: 10 } },
  "Lone Wolf": { pwr: 3, hp: 8, awoken: { pwr: 2, hp: 10 } },
  "Drummer": { pwr: 1, hp: 3, awoken: { pwr: 0, hp: 6 } },
  "Bodyguard": { pwr: 1, hp: 9, awoken: { pwr: 2, hp: 12 } },
  "Chrysalis": { pwr: 0, hp: 3, awoken: { pwr: 0, hp: 6 } },
  "Trickster": { pwr: 2, hp: 7, awoken: { pwr: 2, hp: 12 } },
  "Understudy": { pwr: 2, hp: 17, awoken: { pwr: 2, hp: 12 } },
  "Gravedigger": { pwr: 1, hp: 2, awoken: { pwr: 0, hp: 2 } },
  "Conductor": { pwr: 1, hp: 2, awoken: { pwr: 0, hp: 6 } },
};
