// PWR and HP of the rules v2 pool, written by `npm run mvp:tune-v2`
// (scripts/tune-v2.ts): the numbers come from the tune, never from the
// designer. A unit missing here keeps its row's placeholder numbers.
export const V2_NUMBERS: Record<string, { pwr: number; hp: number; awoken?: { pwr: number; hp: number } }> = {
  "Hourglass": { pwr: 1, hp: 4 },
  "Mirror": { pwr: 2, hp: 16 },
  "Candle": { pwr: 6, hp: 16 },
  "Gambler": { pwr: 1, hp: 3 },
  "Nautilus": { pwr: 1, hp: 3 },
  "Magnet": { pwr: 2, hp: 16 },
  "Raven": { pwr: 1, hp: 9 },
  "Seed": { pwr: 0, hp: 8 },
  "Sniper": { pwr: 3, hp: 16 },
  "Medic": { pwr: 1, hp: 15 },
  "Leech": { pwr: 6, hp: 16 },
  "Metronome": { pwr: 1, hp: 5 },
  "Lone Wolf": { pwr: 3, hp: 6 },
  "Drummer": { pwr: 1, hp: 3 },
  "Bodyguard": { pwr: 1, hp: 8 },
  "Chrysalis": { pwr: 0, hp: 4 },
  "Trickster": { pwr: 2, hp: 6 },
  "Understudy": { pwr: 1, hp: 13 },
  "Gravedigger": { pwr: 1, hp: 3 },
  "Conductor": { pwr: 1, hp: 3 },
};
