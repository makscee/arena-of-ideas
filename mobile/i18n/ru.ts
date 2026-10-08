// The Russian catalog (mission #810, M4-2): every key of en.ts, one file per
// part like the English. Game words match M4-3's word table (src/describe-ru.ts,
// src/glossary-ru.ts): АТК, ОЗ, Пробуждённый, Яд, Щит… "Slayer" is
// «цареубийца» everywhere. catalog.test.ts fails on a key missing here.
import type { Key } from "./en";
import type { Msg } from "./index";
import { battle } from "./ru/battle";
import { codex } from "./ru/codex";
import { main } from "./ru/main";
import { screens } from "./ru/screens";
import { shop } from "./ru/shop";
import { ui } from "./ru/ui";

export const ru = { ...main, ...shop, ...battle, ...codex, ...screens, ...ui } satisfies Record<Key, Msg>;
