// Refusals in plain words (R2-17): the server answers 409 with the run
// engine's reason (src/mvp/run.ts), written for logs; the player reads this.

/** A decision the rules refused (the server's 409, src/mvp/run.ts) in plain,
 * short words: "invalid buy: the line is full" reads "Line full: sell or
 * fuse first". One it doesn't know keeps the server's reason. */
export function plainRefusal(message: string): string {
  const m = message.replace(/^invalid [a-z]+: /, "");
  const gold = /^costs (\d+), have (\d+)/.exec(m);
  if (gold) return `Not enough gold: it costs ${gold[1]}g, you have ${gold[2]}g`;
  // The awakening gift (R3-15/16): picking needs room, and nothing else goes while it waits.
  if (/full: sell a unit to make room, or skip the gift/.test(m)) return "Line and bench full: sell a unit to make room, or skip the gift";
  if (/pick your awakening gift first/.test(m)) return "Pick your gift first, or skip it";
  if (/line and bench are full/i.test(m)) return "Line and bench full: sell or fuse first";
  if (/line is full/i.test(m)) return "Line full: sell or fuse first";
  if (/only the Crown fight is left/.test(m)) return "The Crown: your line is final, only the fight is left";
  if (/the run is over/.test(m)) return "This run is over";
  if (/content is no longer live/.test(m)) return "The game changed since this run began: start a new run";
  if (/no offer in slot/.test(m)) return "That offer is gone: the shop changed";
  return m.charAt(0).toUpperCase() + m.slice(1);
}

/** A refusal on a disabled Buy button, short enough for one line there (the
 * phone sheet's Buy beside Close, the 1024px inspector): "Line full: sell or
 * fuse first" reads "Full: sell or fuse". The error line keeps the long one. */
export function buttonRefusal(text: string): string {
  if (/make room/.test(text)) return "Full: make room";
  return /^Line (and bench )?full/.test(text) ? "Full: sell or fuse" : text;
}
