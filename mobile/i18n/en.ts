// The English catalog (mission #810, M4-1): every piece of player-facing text
// in the client, one file per part of it. Keys are "<screen>.<what>"; {name}
// is a param; a { one, other } message is a plural picked by params.n
// (see index.ts).
import type { Msg } from "./index";
import { battle } from "./en/battle";
import { codex } from "./en/codex";
import { main } from "./en/main";
import { screens } from "./en/screens";
import { shop } from "./en/shop";
import { ui } from "./en/ui";

export const en = { ...main, ...shop, ...battle, ...codex, ...screens, ...ui } satisfies Record<string, Msg>;

export type Key = keyof typeof en;
