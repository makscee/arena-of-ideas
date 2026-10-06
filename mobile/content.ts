// The content pack, fetched once per page (mission #574). Every screen that
// names units, abilities or statuses reads it from here.
import type { MvpContent } from "../src/mvp/contract";
import { api } from "./api";
import { openSummon, setCardAbilities, summonById } from "./ui/card";
import { setUnitRefs } from "./ui/term";

let p: Promise<MvpContent> | null = null;

export const getContent = (): Promise<MvpContent> =>
  (p ??= api.content().then((c) => {
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
  }));
