// The content pack, fetched once per page (mission #574). Every screen that
// names units, abilities or statuses reads it from here.
import type { MvpContent } from "../src/mvp/contract";
import { api } from "./api";

let p: Promise<MvpContent> | null = null;

export const getContent = (): Promise<MvpContent> =>
  (p ??= api.content().catch((e: unknown) => {
    p = null; // a failed load is retried on the next call
    throw e;
  }));
