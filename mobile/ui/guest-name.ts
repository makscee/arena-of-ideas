// A generated name for PLAY (arena#31): word + word + number, like "Amber Fox
// 417", the shape the server's guest names have (server/src/mvp/crazygames.ts).
// It fits the server's name rule (1–24 letters, digits, spaces, _ or -).

const FIRST = ["Amber", "Ash", "Brave", "Bright", "Calm", "Clever", "Copper", "Crimson", "Dawn", "Dusk", "Ember", "Frost", "Gold", "Iron", "Jade", "Lucky", "Moon", "Quiet", "Rapid", "Silver", "Storm", "Sun", "Swift", "Wild"];
const SECOND = ["Badger", "Bear", "Crane", "Drake", "Falcon", "Fox", "Hare", "Hawk", "Heron", "Lynx", "Moth", "Otter", "Owl", "Raven", "Seal", "Stag", "Tiger", "Viper", "Wolf", "Wren"];

/** `rnd(n)` is a whole number in 0..n−1 (Math.random by default). */
export function guestName(rnd: (n: number) => number = (n) => Math.floor(Math.random() * n), digits = 3): string {
  const pick = <T>(xs: readonly T[]): T => xs[rnd(xs.length)]!;
  const top = 10 ** digits;
  return `${pick(FIRST)} ${pick(SECOND)} ${1 + rnd(top - 1)}`;
}
