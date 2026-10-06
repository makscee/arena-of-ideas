// The content pack, fetched once per page (mission #574). Every screen that
// names units, abilities or statuses reads it from here.
import type { MvpContent } from "../src/mvp/contract";
import { api } from "./api";
import { setCardAbilities } from "./ui/card";

let p: Promise<MvpContent> | null = null;

export const getContent = (): Promise<MvpContent> =>
  (p ??= api.content().then((c) => {
    setCardAbilities(c.abilities); // cards draw their When · Who · Does icons from it
    return c;
  }, (e: unknown) => {
    p = null; // a failed load is retried on the next call
    throw e;
  }));
