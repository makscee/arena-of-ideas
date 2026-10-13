/**
 * The word forge's queue on the host (makscee/void-board#1052), against the
 * server's SQLite file.
 *
 *   npm run mvp:words -- request <idea|night|batch> <ref> "<word>" "<part of the design that needed it>"
 *       ref: an idea id, a pair ("unitA+unitB", order dropped) or a unit id
 *   npm run mvp:words -- list          (the Dictionary: every word, its state and its requests)
 *   npm run mvp:words -- card "<word>" (the board card for forging a word: title, then body)
 *   npm run mvp:words -- waiting <unitA> <unitB>   (what a pair is sealed on, if anything)
 *
 * Env: MVP_DB (default data/arena-mvp.db). The forged list is registry/forged-words.json.
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { SqliteMvpStore } from "./sqlite-store.js";
import type { WordSource } from "./store.js";
import { WORD_SOURCES, dictionary, forgedNotFound, pairKey, requestWord, sealedLine, waitingOn, wordKey, type DictionaryEntry } from "./words.js";

const CARD_FILE = fileURLToPath(new URL("../../../docs/words/CARD.md", import.meta.url));

/** The board card for forging a word: the title, and the body from docs/words/CARD.md. */
export function wordCard(e: DictionaryEntry, template = readFileSync(CARD_FILE, "utf8")): { title: string; body: string } {
  const asks = e.requests.map((r) => `- ${r.source} \`${r.ref}\`: ${r.part || "(no text)"} (${r.createdAt.slice(0, 10)})`).join("\n");
  return {
    title: `Arena v2 · word forge: ${e.word}`,
    body: template.replaceAll("{{word}}", e.word).replace("{{requests}}", asks || "- nobody yet"),
  };
}

const line = (e: DictionaryEntry): string => {
  const by = new Set(e.requests.map((r) => r.source));
  const state = e.state === "found" ? `found in ${e.found!.pair} by ${e.found!.playerId}` : e.state === "forged" ? `forged as ${e.kind} (card #${e.card}), not yet found` : "requested";
  return `${e.word}: ${state}${by.size ? ` · ${e.requests.length} request${e.requests.length === 1 ? "" : "s"} (${[...by].join(", ")})` : ""}`;
};

function main(argv: string[], store: SqliteMvpStore): number {
  const [cmd, ...rest] = argv;
  if (cmd === "request") {
    const [source, ref, word, part = ""] = rest;
    if (!WORD_SOURCES.includes(source as WordSource) || !ref || !word) {
      console.error('usage: request <idea|night|batch> <ref> "<word>" "<part>"');
      return 2;
    }
    const added = requestWord(store, source as WordSource, source === "night" && ref.includes("+") ? pairKey(...(ref.split("+") as [string, string])) : ref, { word, part }, new Date());
    console.log(added ? `queued "${word}" from ${source} ${ref}` : `already queued: "${word}" from ${source} ${ref}`);
    return 0;
  }
  if (cmd === "list") {
    const entries = dictionary(store);
    for (const e of entries) console.log(line(e));
    console.log(`${entries.length} words; ${forgedNotFound(entries)} forged, not yet found`);
    return 0;
  }
  if (cmd === "card") {
    const key = wordKey(rest[0] ?? "");
    const e = dictionary(store).find((x) => wordKey(x.word) === key);
    if (!e) {
      console.error(`no word "${rest[0] ?? ""}" in the queue`);
      return 1;
    }
    const { title, body } = wordCard(e);
    console.log(`${title}\n\n${body}`);
    return 0;
  }
  if (cmd === "waiting") {
    const [a, b] = rest;
    if (!a || !b) {
      console.error("usage: waiting <unitA> <unitB>");
      return 2;
    }
    const words = waitingOn(store, pairKey(a, b));
    console.log(words.length ? sealedLine(words) : "not sealed");
    return 0;
  }
  console.error("usage: request | list | card | waiting (see the header of words-cli.ts)");
  return 2;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exit(main(process.argv.slice(2), new SqliteMvpStore(process.env.MVP_DB ?? "data/arena-mvp.db")));
}
