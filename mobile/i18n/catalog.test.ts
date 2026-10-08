// The guard for M4-1 (mission #810): player-facing text lives in the catalog
// (i18n/en.ts), never as a literal in a screen. A string literal outside
// i18n/ that reads like English words fails here; ALLOWED holds the literals
// that look like words but aren't text (selectors, protocol, server reasons).
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";
import { en } from "./en";
import { setLang, t } from "./index";

const root = join(__dirname, "..");

/** Literals that read like words but no player sees as text. */
const ALLOWED = new Set<string>([
  // battle.ts matches the caption words src/mvp/trace.ts and src/glossary.ts
  // build (the game's own words, M4-3), never shows these.
  String.raw`\(\d+ absorbed\)|Chain stopped after\s\d+ steps|Time's up|No room|[Ss]udden death|(?<![\p{L}\d])(?:{})(?![\p{L}\d])|\b(?:PWR|HP)\b|[−+]\d+(?!\d)(?!\s+more)`,
  "Time's up",
]);

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return name === "i18n" || name === "public" || name === "icons" ? [] : files(path);
    return name.endsWith(".ts") && !name.endsWith(".test.ts") && name !== "vite.config.ts" ? [path] : [];
  });
}

/** Every string literal in `src` (comments skipped; a template literal's
 * ${…} reads as "{}", and literals inside it are listed too), with its line. */
export function literals(src: string): { text: string; line: number }[] {
  const out: { text: string; line: number }[] = [];
  let i = 0;
  const lineAt = (at: number) => src.slice(0, at).split("\n").length;
  const scan = (stopAtBrace: boolean): void => {
    let depth = 0;
    while (i < src.length) {
      const c = src[i];
      if (c === "/" && src[i + 1] === "/") { i = src.indexOf("\n", i); if (i < 0) i = src.length; continue; }
      if (c === "/" && src[i + 1] === "*") { i = src.indexOf("*/", i + 2) + 2; continue; }
      if (stopAtBrace && c === "{") depth++;
      if (stopAtBrace && c === "}") { if (depth === 0) { i++; return; } depth--; }
      if (c === '"' || c === "'") {
        const start = i++;
        let text = "";
        while (i < src.length && src[i] !== c) { if (src[i] === "\\") { text += src[i + 1]; i += 2; } else text += src[i++]; }
        i++;
        out.push({ text, line: lineAt(start) });
        continue;
      }
      if (c === "`") {
        const start = i++;
        let text = "";
        while (i < src.length && src[i] !== "`") {
          if (src[i] === "\\") { text += src[i + 1]; i += 2; }
          else if (src[i] === "$" && src[i + 1] === "{") { i += 2; text += "{}"; scan(true); }
          else text += src[i++];
        }
        i++;
        out.push({ text, line: lineAt(start) });
        continue;
      }
      i++;
    }
  };
  scan(false);
  return out;
}

/** Reads like text for a player: a capitalised word then a word ("Sell or
 * fuse"), or words ending a sentence ("is over."). */
export function looksLikeText(s: string): boolean {
  return /\b[A-Z][a-z']+,? [a-z{]/.test(s) || /[a-z]{2,} [a-z]{2,}[.!?]$/.test(s) || /^[A-Z][a-z]+[.!?]$/.test(s);
}

describe("client strings live in the catalog", () => {
  it("no screen has an English sentence outside i18n/", () => {
    const found: string[] = [];
    for (const file of files(root)) {
      for (const { text, line } of literals(readFileSync(file, "utf8"))) {
        if (looksLikeText(text) && !ALLOWED.has(text)) found.push(`${relative(root, file)}:${line} ${JSON.stringify(text)}`);
      }
    }
    expect(found).toEqual([]);
  });

  it("t fills params and picks plurals", () => {
    expect(t("test.ideas" as never, { n: 1 })).toBe("test.ideas");
    setLang("ru", { ["test.ideas" as never]: { one: "{n} идея", few: "{n} идеи", many: "{n} идей", other: "{n} идеи" } });
    expect([1, 2, 5, 21].map((n) => t("test.ideas" as never, { n }))).toEqual(["1 идея", "2 идеи", "5 идей", "21 идея"]);
    setLang("en", {});
  });

  it("every catalog message is non-empty", () => {
    for (const [key, msg] of Object.entries(en)) {
      const forms = typeof msg === "string" ? [msg] : Object.values(msg);
      for (const form of forms) expect(form, key).not.toBe("");
    }
  });
});
