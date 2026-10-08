// The content pack (mission #574). Every screen that names units, abilities
// or statuses reads it from here. It is fetched once and kept while
// /health's contentVersion stays the same; when the pool changes (M2-2) the
// next screen that asks gets the new pack, without a reload.
import type { MvpContent, UnitContent } from "../src/mvp/contract";
import { api } from "./api";
import { openSummon, setCardAbilities, summonById } from "./ui/card";
import { setUnitRefs } from "./ui/term";

/** How often, at most, a screen's getContent() asks /health for the version. */
const CHECK_MS = 15_000;

let p: Promise<MvpContent> | null = null;
let checkedAt = 0;

const load = (): Promise<MvpContent> =>
  api.content().then((c) => {
    checkedAt = Date.now();
    setCardAbilities(c.abilities); // cards draw their When · Who · Does icons from it
    // "Imp (1/2)" in a unit's text opens the Imp's card (R3-5).
    setUnitRefs({
      emoji: (id) => summonById(c, id)?.emoji,
      open: (id) => {
        const s = summonById(c, id);
        if (s) openSummon(s, c);
      },
    });
    return c;
  }, (e: unknown) => {
    p = null; // a failed load is retried on the next call
    throw e;
  });

/** The content pack; refetched when /health says the pool changed (checked
 * at most every CHECK_MS). A failed check keeps the pack it has. */
export async function getContent(): Promise<MvpContent> {
  if (p && Date.now() - checkedAt > CHECK_MS) {
    checkedAt = Date.now();
    const [have, health] = await Promise.all([p.catch(() => null), api.health().catch(() => null)]);
    if (have && health?.contentVersion && health.contentVersion !== have.version) p = null;
  }
  return (p ??= load());
}

/** A unit by id: the live pool's, else one that has left it (a champion's
 * line, a run on an older pool, a fusion's part, M2-2). */
export function unitIn(content: MvpContent, id: string): UnitContent | undefined {
  return content.units.find((x) => x.id === id) ?? content.left?.find((x) => x.id === id);
}
