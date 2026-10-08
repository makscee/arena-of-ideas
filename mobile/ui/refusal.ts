// Refusals in plain words (R2-17): the server answers 409 with the run
// engine's reason (src/mvp/run.ts), written for logs; the player reads this.
import { t } from "../i18n";

/** A decision the rules refused (the server's 409, src/mvp/run.ts) in plain,
 * short words: "invalid buy: the line is full" reads "Line full: sell or
 * fuse first". One it doesn't know keeps the server's reason. */
export function plainRefusal(message: string): string {
  const m = message.replace(/^invalid [a-z]+: /, "");
  const gold = /^costs (\d+), have (\d+)/.exec(m);
  if (gold) return t("refusal.gold", { cost: gold[1]!, have: gold[2]! });
  // The awakening gift (R3-15/16): picking needs room, and nothing else goes while it waits.
  if (/full: sell a unit to make room, or skip the gift/.test(m)) return t("refusal.giftFull");
  if (/pick your awakening gift first/.test(m)) return t("refusal.giftFirst");
  if (/line and bench are full/i.test(m)) return t("refusal.bothFull");
  if (/line is full/i.test(m)) return t("refusal.lineFull");
  if (/only the Crown fight is left/.test(m)) return t("refusal.crownOnly");
  if (/the run is over/.test(m)) return t("refusal.runOver");
  if (/content is no longer live/.test(m)) return t("refusal.contentChanged");
  if (/no offer in slot/.test(m)) return t("refusal.offerGone");
  // One it doesn't know: the server's own words (M4-2 note: English only).
  return m.charAt(0).toUpperCase() + m.slice(1);
}

/** A refusal on a disabled Buy button, short enough for one line there (the
 * phone sheet's Buy beside Close, the 1024px inspector): "Line full: sell or
 * fuse first" reads "Full: sell or fuse". The error line keeps the long one. */
export function buttonRefusal(text: string): string {
  if (text === t("refusal.giftFull")) return t("refusal.buttonMakeRoom");
  return text === t("refusal.lineFull") || text === t("refusal.bothFull") ? t("refusal.buttonFull") : text;
}
