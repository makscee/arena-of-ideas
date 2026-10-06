import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { afterEach, describe, expect, it } from "vitest";
import type { Champion, DecisionResponse, FusionDiscovery, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { mvpContent } from "./content.js";
import { hasCrudeStem } from "./crude.js";
import { BENCH_NAMES, CRUDE_BEFORE_594, GLUED_STEMS, INNOCENT_NAMES, LIVE_NAMES, MASK_INNOCENT, MASK_LEAKS } from "./namer-corpus.js";
import { awaitFusionName, cleanModelName, drainFusionNames, hasUnitName, NAMER_LETTERS, isBlockedName, fusionNameReady, fusionNaming, httpModelNamer, MODEL_DOWN_MS, NAMER_EXAMPLES, MODEL_FAILURES, MODEL_PROBE_MS, portmanteau, recordFusion, storedOrPortmanteau, type ModelNamer } from "./fusions.js";
import { decide, preview, startRun } from "./runs.js";
import { seedChampion } from "./bots.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

const form = { when: [], who: [], does: ["a"] };
const unit = (id: string, name: string): UnitContent => ({ id, name, emoji: "x", tier: 1, base: { pwr: 1, hp: 1 }, forms: { sleeping: form, awoken: form } });
const brawler = unit("brawler", "Brawler");
const medic = unit("medic", "Medic");
const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const bot: PlayerRef = { id: "bot", name: "bot-Ash", bot: true };
/** Distinct model names: pairs never share one, so a stub answers each ask with the next. */
const NAMES = ["Duskmend", "Gloomfang", "Ashwarden", "Briarhusk", "Cindermaw", "Dreadbloom", "Emberlash", "Frostgrin", "Galeborn", "Hollowtusk", "Ironquill", "Jadecoil"];

describe("MVP fusion names: the fallback and the store", () => {
  it("falls back to a deterministic portmanteau", () => {
    expect(portmanteau("Brawler", "Medic")).toBe("Brawdic");
    // One word, title-cased: no inner capital from a two-word name.
    expect(portmanteau("War Drummer", "Medic")).toBe("Wardrdic");
    expect(portmanteau("Brawler", "Medic")).toBe(portmanteau("Brawler", "Medic"));
    expect(portmanteau("Medic", "Brawler")).not.toBe(portmanteau("Brawler", "Medic"));
  });

  it("never falls back to a part's name or a base unit's name", () => {
    const names = mvpContent().units.map((u) => u.name);
    expect(portmanteau("Rose", "Rot")).not.toBe("Rot");
    expect(portmanteau("Rat", "Gnat", names)).not.toBe("Rat");
    const folded = new Set(names.map((x) => x.toLowerCase().replace(/[^a-z]/g, "")));
    for (const x of names) for (const y of names) if (x !== y) expect(folded.has(portmanteau(x, y, names).toLowerCase()), `${x}+${y}`).toBe(false);
  });

  it("credits a human, never a bot, and keeps a stored name and credit", () => {
    const store = new MemoryMvpStore();
    const name = storedOrPortmanteau(store);
    expect(name(brawler, medic, maks)).toEqual({ name: "Brawdic", discoveredBy: maks });
    expect(name(brawler, medic, bot)).toEqual({ name: "Brawdic", discoveredBy: null });

    const byBot: FusionDiscovery = { first: "brawler", second: "medic", name: "Ironcare", discoveredBy: null, discoveredAt: "t", nameSource: "model" };
    store.putFusion(byBot);
    expect(name(brawler, medic, maks)).toEqual({ name: "Ironcare", discoveredBy: maks });
    store.putFusion({ ...byBot, discoveredBy: maks });
    expect(name(brawler, medic, { id: "p2", name: "Eva", bot: false })).toEqual({ name: "Ironcare", discoveredBy: maks });
    expect(store.fusion("medic", "brawler")).toBeUndefined();
    expect(store.fusions()).toHaveLength(1);
  });

  it("without a model, decide's namer and the preview's agree on the portmanteau", () => {
    const store = new MemoryMvpStore();
    const naming = fusionNaming({ store, content: { version: "t", units: [brawler, medic], abilities: {}, statuses: {} }, now: () => new Date(0) }, { model: null });
    expect(naming.nameFusion(brawler, medic, maks)).toEqual(naming.peek(brawler, medic, maks));
    expect(naming.peek(brawler, medic, maks)).toEqual({ name: "Brawdic", discoveredBy: maks });
  });
});

describe("MVP fusion names: every pair's name is its own", () => {
  const content = mvpContent();
  const u = (id: string) => content.units.find((x) => x.id === id)!;
  const fold = (x: string) => x.toLowerCase().replace(/[^a-z0-9]/g, "");

  it("the portmanteau moves the split off a taken name, then any split, then a numeral", () => {
    expect(portmanteau("Silencer", "Bat")).toBe("Sileat");
    expect(portmanteau("Silencer", "Gnat", ["Sileat"])).not.toBe("Sileat");
    expect(portmanteau("Spike", "Squire", ["SPIRE"])).not.toBe("Spire");
    // Nothing left but a numeral: every split and the whole join are taken.
    const all = ["Ab", "Abd", "Ad", "Acd", "Abcd", "Ad", "A", "Acd"];
    expect(portmanteau("Ab", "Cd", all)).toBe("Ad II");
    expect(portmanteau("Ab", "Cd", [...all, "ad ii"])).toBe("Ad III");
  });

  it("with no model, pairs that share a portmanteau get different names, and stored names never change", () => {
    const store = new MemoryMvpStore();
    // Two pairs stored with one name before this rule keep it.
    store.putFusion({ first: "spike", second: "wire", name: "Spire", discoveredBy: null, discoveredAt: "t", nameSource: "fallback" });
    store.putFusion({ first: "spike", second: "squire", name: "Spire", discoveredBy: null, discoveredAt: "t", nameSource: "fallback" });
    const naming = fusionNaming({ store, content, now: () => new Date(0) }, { model: null });
    const fuse = (a: string, b: string) => recordFusion(store, { first: a, second: b, ...naming.nameFusion(u(a), u(b), maks) }, "t").name;
    expect(naming.peek(u("spike"), u("wire"), maks).name).toBe("Spire");
    expect(naming.peek(u("spike"), u("squire"), maks).name).toBe("Spire");
    const names = [fuse("silencer", "bat"), fuse("silencer", "gnat"), fuse("silencer", "rat"), fuse("spike", "gardener")];
    expect(names[0]).toBe("Sileat");
    expect(new Set(names.map(fold)).size).toBe(names.length);
    const units = new Set(content.units.map((x) => fold(x.name)));
    for (const name of names) expect(units.has(fold(name)), name).toBe(false);
    expect(names).not.toContain("Spire");
  });

  it("a model answer another pair has (any case) or a base unit's name is refused: asked again, then the unique portmanteau", async () => {
    const store = new MemoryMvpStore();
    store.putFusion({ first: "rose", second: "victim", name: "Bloom", discoveredBy: null, discoveredAt: "t", nameSource: "model" });
    const answers = ["bloom", "GNAT", "Thornveil"];
    let asked = 0;
    const naming = fusionNaming({ store, content, now: () => new Date(0) }, { model: async () => answers[asked++] ?? "Bloom" });
    naming.hooks.onDecision!(undefined as never, undefined as never, { phase: "shop", player: maks, line: [lineUnitOf(u("rose"), "u1", 3), lineUnitOf(u("spike"), "u2", 3)] } as never);
    await drainFusionNames(store);
    expect(naming.peek(u("rose"), u("spike"), maks).name).toBe("Thornveil");
    // spike+rose: "Bloom" three times, then the portmanteau.
    expect(asked).toBe(6);
    expect(naming.peek(u("spike"), u("rose"), maks).name).toBe(portmanteau("Spike", "Rose", [...content.units.map((x) => x.name), "Bloom"]));
    // Two pairs prepared one name can't both store it: the second fuse falls back.
    expect(naming.peek(u("rose"), u("victim"), maks).name).toBe("Bloom");
  });

  it("a prepared name stored by another pair since falls back to the portmanteau", async () => {
    const store = new MemoryMvpStore();
    const naming = fusionNaming({ store, content, now: () => new Date(0) }, { model: async () => "Grimward" });
    naming.hooks.onDecision!(undefined as never, undefined as never, { phase: "shop", player: maks, line: [lineUnitOf(u("rose"), "u1", 3), lineUnitOf(u("spike"), "u2", 3)] } as never);
    await drainFusionNames(store);
    expect(naming.peek(u("rose"), u("spike"), maks).name).toBe("Grimward");
    store.putFusion({ first: "bat", second: "rat", name: "GRIMWARD", discoveredBy: null, discoveredAt: "t", nameSource: "fallback" });
    const late = fusionNaming({ store, content, now: () => new Date(0) }, { model: null });
    expect(late.peek(u("rose"), u("spike"), maks).name).toBe(portmanteau("Rose", "Spike"));
  });
});

describe("MVP fusion names: the model's answer through the blocklist", () => {
  it("keeps short invented names, title-cased", () => {
    expect(cleanModelName("Ironcare")).toBe("Ironcare");
    expect(cleanModelName('Name: "iron mender".\nIt combines…')).toBe("Iron Mender");
    expect(cleanModelName("<think>hmm</think>\nBloodmend")).toBe("Bloodmend");
    expect(cleanModelName("Ash-Warden")).toBe("Ash-Warden");
    expect(cleanModelName("BalanceWarrior")).toBe("Balance Warrior");
    expect(cleanModelName("KIngard")).toBe("Kingard");
    expect(cleanModelName("The Ashen")).toBe("The Ashen");
    expect(cleanModelName("🥊🎯 Stormancer!")).toBe("Stormancer");
  });

  it("refuses franchise names, fake official titles, spam and non-names", () => {
    for (const raw of ["Pikachu", "Darth Medic", "Gandalf", "Super Mario", "Elsa", "Sonic", "Nefarian", "Swindle", "Admin Approved", "FREE V-BUCKS", "Frozen", "Hollow Knight", "Kel'Thuzad"])
      expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["", "a", "This is a very long name for a unit", "Iron care mend heal", "monarch Of senses", "Iron Care Mend", "Name42", "www scam example", "Ye Ye"]) expect(cleanModelName(raw), raw).toBeNull();
    expect(cleanModelName("Medic", brawler, medic)).toBeNull();
    // Ordinary words that are also franchise names pass inside a longer name.
    expect(cleanModelName("Cloud Warden")).toBe("Cloud Warden");
  });

  it("matches the blocklist after folding: glued, inflected and accented forms", () => {
    for (const raw of ["Darthvader", "Gandalfs", "Marios", "Hogwart", "Pikachú", "Pikach\u0075\u0301", "Thor's Edge", "SkyWalker", "Gul'dan", "Zélda", "Iron Hulks"])
      expect(cleanModelName(raw), raw).toBeNull();
    // Short names are matched per word, so ordinary words that contain them pass.
    for (const raw of ["Thornback", "Invader", "Marionette", "Smuggler", "Scamper", "Supersonic", "Subterran Ox"]) expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("blocks profanity, slurs and hate words, and keeps the ordinary words that contain them (#587)", () => {
    for (const raw of ["Shitlord", "Fuckwit", "Kike", "Retard", "Hitler", "Rapeblade", "Cumlord", "Asshole", "Ass", "Sex", "Pedophile", "Wetback", "Nig Nog", "Redskin", "Beaner", "Dumbass", "Kyke", "Whitepower", "Jewkiller", "Pussies", "Mongoloid", "Pedomancer", "Kill All Jews", "Jew Slayer", "Ching Chong", "Sexwraith", "Fagmancer", "Clitoris"])
      expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["Cockatoo", "Therapist", "Parapet", "Raccoon", "Cocoon", "Assassin", "Glass Golem", "Mongoose", "Scunthorpe", "Coarse", "Hearse", "Cockney", "Torpedo", "Sextant", "Jewel Golem", "Pediatric Ward"])
      expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("a stand-in name is never a blocked word: King then Spike isn't a slur, and no pair of the content gives one (#587)", () => {
    const units = mvpContent().units;
    const names = units.map((u) => u.name);
    expect(isBlockedName(portmanteau("King", "Spike", names))).toBe(false);
    const taken: string[] = [];
    for (const a of units)
      for (const b of units) {
        if (a.id === b.id) continue;
        const name = portmanteau(a.name, b.name, [...names, ...taken]);
        taken.push(name);
        expect(isBlockedName(name), `${a.name}+${b.name}=${name}`).toBe(false);
        // Held stricter than a model's answer: no crude stem anywhere, masks ignored.
        expect(hasCrudeStem(name), `${a.name}+${b.name}=${name}`).toBe(false);
      }
    // #609: "jew" is a stem anywhere again, so no "Injewark".
    expect(portmanteau("Injector", "Bulwark", names)).toBe("Injelwark");
  }, 30_000);

  it("splits CamelCase before matching words, and stems don't over-block ordinary names", () => {
    for (const raw of ["LordVader", "SuperMario", "BabyYoda", "IronMan", "Xmen", "Witchers"]) expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["Hexmender", "Badminton", "Iron Mantle", "Twitcher", "Exterminator", "Carambola", "Fluffy", "Cockatrice", "Dickens", "Scunthorpe", "Cumulus"]) expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("catches lowercase joins, and never matches a stem across a word's edge", () => {
    for (const raw of ["Supermario", "Lordvader", "Babyyoda", "Thorhammer", "Hulkbuster", "Yodaling", "Fusion", "Fused Medic", "Spider Man Medic", "Lordbatman", "Darthawk", "Mind Flayer", "Wrathion", "Ratbert", "Smutnarid", "Shitlord", "Rapeblade", "Cockwraith", "Analbeast", "Penis Golem"])
      expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["Aqua Mantis", "Spider Mantis", "Bat Mantle", "Batmancer", "Dart Hawk", "Sonic Shrieker", "Grim Joker", "Shadow Swindle", "Confusion", "Hulking Brute", "Evader"])
      expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("refuses a crude fragment anywhere, also hidden in an ordinary word: only a whole ordinary word passes", () => {
    // A crude name stays in the world for good; a needless refusal costs one more ask.
    for (const raw of [
      // glued behind a word, or hidden in an ordinary word behind one
      "Noctscum", "Noctscumling", "Noct Scum", "Scumscum", "Scum", "Scumlord", "Widow'sbloom",
      "Kingrape", "Hagrape", "Mindrape", "Gangrape", "Bloodrape", "Deadrape", "Godrape", "Horserape", "Stingrape", "Fangrape",
      "Buttcanal", "Butt Canal", "Ballswank", "Ball Swank", "Wingrape", "Bugrape", "Drugrape", "Voidrape", "Dreadrape", "Lordrape",
      "Swordrape", "Toxicrape", "Roserape", "Kingswank", "Medicrape", "Scytherapist", "Cuminjector",
      // whole-word lists missed these
      "Kingdong", "Kingboner", "Assrat", "Semenrat", "Spermrat", "Buttinjector", "Testiclerat", "Bigcocky", "Gaykiller", "Homokiller",
      "Gypsykiller", "Incestborn", "Orgyking", "King Dong", "Kingbutt", "Asses",
      // ordinary compounds no longer rescued
      "Jolttherapist", "Soultherapist", "Vinegrape", "Bonescrape", "Mistdrape", "Ashgrape", "Firepeacock", "Stormcanal", "King Grape",
      "Viscum", "Glasscumber", "Pussycat",
    ])
      expect(cleanModelName(raw), raw).toBeNull();
    // Innocent words cover a stem, a whole real word passes, and the ordinary
    // words that start or end with ass, butt or dong.
    for (const raw of ["Assassin", "Assault", "Butterfly", "Button", "Buttress", "Nightshade", "Knightfall", "Jewelwing", "Cockatrice", "Mosscumulus",
      "Raccoon", "Grape", "Therapist", "Canal", "Sexton", "Glass", "Brass", "Cutlass", "Hourglass", "Compass", "Glass Golem", "Invader",
      "Grapeshot", "Encumber", "Analyst", "Manaleech", "Farseer", "Starseed", "Spices", "Pedometer", "Peacock", "Cumin"])
      expect(cleanModelName(raw), raw).not.toBeNull();
    // A part's name changes nothing: "Rosegrape" from Rose and Rot is refused too.
    const units = new Map(mvpContent().units.map((u) => [u.name, u]));
    const [rose, rot] = [units.get("Rose")!, units.get("Rot")!];
    for (const raw of ["Roserape", "Rosegrape"]) expect(cleanModelName(raw, rose, rot), raw).toBeNull();
  });

  it("refuses LDNOOBW's words and slurs glued to a unit's name, and fragments that only look like one word (#594, last round)", () => {
    for (const raw of [
      "Paedoking", "Milfnurse", "Gookrat", "Pakirat", "Yidrat", "Klanlord", "Lynchking", "Queerkiller", "Fukking", "Hookerhag", "Hornyfang",
      "Japslayer", "Krautkiller", "Wiggerfang", "Lesbofang", "Hentaiwing", "Spunkrat", "Cuckfang",
      "Sexagen", "Sextup", "Clitter", "Starse", "Farse", "Arsen", "Pedolog", "Kingtit", "Semendrake", "Niggard", "Rapeseed",
    ])
      expect(cleanModelName(raw), raw).toBeNull();
  });

  it("refuses every crude entry of the lists before #594, alone and glued mid-word (#609)", () => {
    // The short ambiguous stems are refused only at a word's edge; abo only at its end.
    const edge = new Set(["ass", "asses", "butt", "dong", "jap", "cuck", "thot", "fap", "horny", "abo"]);
    for (const word of CRUDE_BEFORE_594) {
      expect(isBlockedName(word[0]!.toUpperCase() + word.slice(1)), word).toBe(true);
      if (!edge.has(word)) expect(isBlockedName(`King${word}rat`), `King${word}rat`).toBe(true);
    }
  });

  // Common heads, and heads ending in a, e, r or u: a mask built on a head's
  // last letters ("rshti", "ashta", "among") would hide a stem glued behind it.
  const glueHeads = ["", "King", "Dark", "Moon", "Bone", "Ember", "Aura", "Lunar", "Terra", "Ura"];
  const glueTails = ["rat", "drake", "head", "horn", "demon", "fang", "lord", "reaper"];
  const glued = (head: string, word: string, tail: string) => (head ? `${head}${word}${tail}` : word[0]!.toUpperCase() + word.slice(1) + tail);

  it("no mask lets a pre-#594 entry through when glued to common heads and tails (R2-17: 'kland' let Klandrake pass)", () => {
    const edge = new Set(["ass", "asses", "butt", "dong", "jap", "cuck", "thot", "fap", "horny", "abo"]);
    const passed: string[] = [];
    for (const word of CRUDE_BEFORE_594) {
      if (edge.has(word)) continue;
      for (const head of glueHeads) for (const tail of glueTails) if (!isBlockedName(glued(head, word, tail))) passed.push(glued(head, word, tail));
    }
    expect(passed).toEqual([]);
  });

  it("no mask lets a toilet word or a short stem through glued to a head ending in a vowel or r, and the masks' innocent words pass (R2-17)", () => {
    // bum and knob are refused only at a word's edge, so only as a head or a tail.
    const edge = new Set(["bum", "knob"]);
    const passed: string[] = [];
    for (const word of GLUED_STEMS)
      for (const head of glueHeads)
        for (const tail of glueTails) {
          const names = edge.has(word) ? [glued(head, word, ""), glued("", word, tail)] : [glued(head, word, tail)];
          for (const name of names) if (name && !isBlockedName(name)) passed.push(name);
        }
    expect(passed).toEqual([]);
    expect(MASK_LEAKS.filter((name) => !isBlockedName(name))).toEqual([]);
    expect(MASK_INNOCENT.filter((name) => isBlockedName(name))).toEqual([]);
  });

  it("refuses the regressions, the stems that left EDGE, LDNOOBW's words back from SKIP, and toilet words (#609)", () => {
    for (const raw of [
      "Piss", "Pissrat", "Kingpiss", "Pisser", "Chinaman", "Coolie", "Abo", "Kingabo", "Abos",
      "Kingklansman", "Killjewrat", "Kingassface", "Libtardrat", "Kinglynch", "Kinggookrat", "Kingcoonrat", "Kinghomorat", "Kingpoofrat",
      "Kinganusrat", "Kingqueerrat",
      "Nudewing", "Eroticfang", "Kinkyrat", "Toplessrat", "Bustyfang", "Nympholord", "Bondagelord", "Gropefang", "Swingerrat", "Escortrat",
      "Lolitawing", "Intercourse", "Undressing",
      "Fartking", "Kingfart", "Poopling", "Crapfang", "Bumrat", "Kingbum", "Pimpking", "Pervlord", "Gimpfang", "Knobrat", "Kingknob",
      "Erectwing", "Hymenrat",
    ])
      expect(cleanModelName(raw), raw).toBeNull();
    // Abo and the short ambiguous stems pass inside a word, and masks excuse the rest.
    for (const raw of ["About", "Above", "Abomination", "Album", "Bumblebee", "Knobble", "Scrapper", "Bonebuster"])
      expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("passes every innocent fantasy name of the corpus and the live world, and refuses at most 3% of the bench (#594)", () => {
    // Collateral is measured: a corpus name refused needs a mask in crude.ts.
    for (const name of [...INNOCENT_NAMES, ...LIVE_NAMES]) expect(isBlockedName(name), name).toBe(false);
    for (const raw of ["Stardust", "Stardrake", "Bonereaper", "Manalith", "Spicefang", "Cuirass", "Embarrass", "Benign", "Moonignite", "Mustardseed",
      "Sauerkraut", "Pakistan", "Yiddish", "Japan", "Cuckoo-clock", "Thornyfang", "Titanfang", "Mongoose"])
      expect(cleanModelName(raw), raw).not.toBeNull();
    const refused = BENCH_NAMES.filter((n) => isBlockedName(n));
    expect(refused).toEqual(["Jolttherapist", "Nighbattler", "Nighflare", "Shadowskeet"]);
    expect(refused.length / BENCH_NAMES.length).toBeLessThanOrEqual(0.03);
  });

  it("takes an apostrophe only as a possessive before a second word", () => {
    for (const raw of ["Widow'sbloom", "Kel'thas", "O'Bloom", "Widow's"]) expect(cleanModelName(raw), raw).toBeNull();
    expect(cleanModelName("Widow's Bloom")).toBe("Widow's Bloom");
  });

  it("finds a unit's whole name of 5+ letters inside a name", () => {
    const harvest = unit("harvest", "Harvest");
    const rat = unit("plague", "Plague Rat");
    const king = unit("king", "King");
    for (const name of ["Harvestguard", "Blood Harvest", "Plagueratling"]) expect(hasUnitName(name, [harvest, rat, king]), name).toBe(true);
    for (const name of ["Kingsbane", "Plaguebite", "Harvguard"]) expect(hasUnitName(name, [harvest, rat, king]), name).toBe(false);
  });

  it("refuses names that only join the two parts", () => {
    const squire = unit("squire", "Squire");
    const gnat = unit("gnat", "Gnat");
    for (const raw of ["SquireGnat", "Gnat Squire", "Squire-Gnat", "Squires", "Gnat"]) expect(cleanModelName(raw, squire, gnat), raw).toBeNull();
    expect(cleanModelName("Squat", squire, gnat)).toBe("Squat");
    expect(cleanModelName("Gnatling Warden", squire, gnat)).toBe("Gnatling Warden");
    const golem = unit("golem", "Iron Golem");
    expect(cleanModelName("GolemGnat", golem, gnat)).toBeNull();
  });
});

describe("MVP fusion names: through the runtime", () => {
  const content = mvpContent();
  const [a, b, c] = content.units;

  /** A runtime whose namer asks `model`, with a human run holding a, b and c Awoken. */
  function world(model: ModelNamer | null, player: PlayerRef = maks, backoffMs?: (failures: number) => number, clock?: () => number) {
    const rt = mvpRuntime({ content, seed: () => 7 });
    // Swap in a namer with this model (the runtime's own reads ARENA_NAMER_URL).
    const naming = fusionNaming(rt, { model, ...(backoffMs ? { backoffMs } : {}), ...(clock ? { clock } : {}) });
    rt.hooks[0] = naming.hooks;
    rt.nameFusion = naming.nameFusion;
    rt.peekFusionName = naming.peek;
    const run = startRun(rt, player);
    rt.store.putRun({ ...run, line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 3, rt.rules), lineUnitOf(c!, "u3", 3, rt.rules)], nextUid: 4 });
    const stored = () => rt.store.run(run.runId)!;
    return { rt, runId: run.runId, stored };
  }

  it("prefetches model names for every fusable ordered pair, and the first fuse stores the model's name with the credit", async () => {
    const asked: string[] = [];
    const { rt, stored } = world(async (x, y) => (asked.push(`${x.id}+${y.id}`), `${x.name.slice(0, 3)}${y.name.slice(-3)}wyn`));
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(asked).toHaveLength(6);

    // The prepared name stays hidden until the fuse.
    const pv = preview(rt, stored(), { kind: "fuse", first: 1, second: 0 });
    const want = cleanModelName(`${b!.name.slice(0, 3)}${a!.name.slice(-3)}wyn`)!;
    expect(pv.run.line[0]).toMatchObject({ kind: "fused", name: "", fusion: { name: "", discoveredBy: maks } });
    const done: DecisionResponse = decide(rt, stored(), { kind: "fuse", first: 1, second: 0 });
    expect(done.run.line[0]).toMatchObject({ name: want, fusion: { name: want, discoveredBy: maks } });
    expect(rt.store.fusions()).toEqual([{ first: b!.id, second: a!.id, name: want, discoveredBy: maks, discoveredAt: expect.any(String), nameSource: "model" }]);

    // The same pair in the same order gets the same name again, for anyone; the credit stays.
    const other = world(null, { id: "p2", name: "Eva", bot: false });
    const eva = fusionNaming({ ...other.rt, store: rt.store }, { model: null });
    expect(eva.peek(b!, a!, { id: "p2", name: "Eva", bot: false })).toEqual({ name: want, discoveredBy: maks });
  });

  it("a preview shows a pair bots made with its name and no credit; the human's fuse claims it", () => {
    const { rt, stored } = world(null);
    const name = portmanteau(a!.name, b!.name);
    rt.store.putFusion({ first: a!.id, second: b!.id, name, discoveredBy: null, discoveredAt: "2026-10-06T00:00:00.000Z", nameSource: "fallback" });
    const pv = preview(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    expect(pv.run.line[0]).toMatchObject({ name, fusion: { name, discoveredBy: null } });
    // The other order is a pair nobody has fused.
    expect(preview(rt, stored(), { kind: "fuse", first: 1, second: 0 }).run.line[0]).toMatchObject({ name: "", fusion: { name: "" } });
    const done = decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    expect(done.run.line[0]).toMatchObject({ name, fusion: { name, discoveredBy: maks } });
  });

  it("with the model down, fuses at once with the portmanteau, and a human's fuse fixes it for good", async () => {
    let down = true;
    const asked: string[] = [];
    const { rt, stored } = world(async (x, y) => {
      asked.push(`${x.id}+${y.id}`);
      if (down) throw new Error("connection refused");
      return "Grimward";
    }, maks, () => 0);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(asked).toHaveLength(6);
    const done = decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    const name = portmanteau(a!.name, b!.name);
    expect(done.run.line[0]).toMatchObject({ name, fusion: { name, discoveredBy: maks } });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name, nameSource: "fallback", discoveredBy: maks });
    // The model comes back: the failed pairs are asked again, except the one a human fused.
    down = false;
    asked.length = 0;
    await drainFusionNames(rt.store);
    expect(asked).not.toContain(`${a!.id}+${b!.id}`);
    expect(asked.length).toBeGreaterThan(0);
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name, nameSource: "fallback" });
    expect(stored().line[0]).toMatchObject({ name, fusion: { name } });
  });

  it("asks again with a backoff when the model fails, until it answers", async () => {
    let calls = 0;
    const waits: number[] = [];
    const { rt, stored } = world(async () => {
      calls++;
      if (calls <= 2) throw new Error("timeout");
      return "Grimward";
    }, maks, (failures) => (waits.push(failures), 0));
    rt.store.putRun({ ...stored(), line: stored().line.slice(0, 2) });
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // queues a+b and b+a
    await drainFusionNames(rt.store); // both pairs fail once
    await drainFusionNames(rt.store); // and are asked again: both answer
    expect(waits).toEqual([1, 1]);
    const done = decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    expect(done.run.line[0]).toMatchObject({ name: "Grimward" });
  });

  it("the first fuse fixes a pair's name for good, a bot's too: a later model answer changes nothing", async () => {
    let release!: (name: string) => void;
    const answered = new Promise<string>((r) => (release = r));
    const { rt, stored } = world(() => answered, bot);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // queues the bot's pairs
    const draining = drainFusionNames(rt.store);
    // Fused before the model answered: the fallback, for good.
    decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    const fallback = portmanteau(a!.name, b!.name);
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: fallback, nameSource: "fallback", discoveredBy: null });
    release("Grimward");
    await draining;
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: fallback, nameSource: "fallback" });
    expect(stored().line[0]).toMatchObject({ name: fallback, fusion: { name: fallback } });
    // A human fusing it later gets the same name and claims the credit.
    const human = startRun(rt, maks);
    rt.store.putRun({ ...human, line: [lineUnitOf(a!, "h1", 3, rt.rules), lineUnitOf(b!, "h2", 3, rt.rules)], nextUid: 3 });
    const done = decide(rt, rt.store.run(human.runId)!, { kind: "fuse", first: 0, second: 1 });
    expect(done.run.line[0]).toMatchObject({ name: fallback, fusion: { discoveredBy: maks } });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: fallback, discoveredBy: maks });
  });

  it("asks for humans' pairs before bots', also pairs queued while it drains", async () => {
    const asked: string[] = [];
    const botSide = world(async (x, y) => (asked.push(`${x.id}+${y.id}`), NAMES[asked.length]!), bot);
    const { rt } = botSide;
    decide(rt, botSide.stored(), { kind: "reorder", from: 0, to: 0 }); // 6 bots' pairs of a, b, c
    const human = startRun(rt, maks);
    rt.store.putRun({ ...human, line: [lineUnitOf(c!, "h1", 3, rt.rules), lineUnitOf(a!, "h2", 3, rt.rules)], nextUid: 3 });
    decide(rt, rt.store.run(human.runId)!, { kind: "reorder", from: 0, to: 0 }); // c+a and a+c move up
    await drainFusionNames(rt.store);
    expect(asked).toHaveLength(6);
    expect(asked.slice(0, 2).sort()).toEqual([`${a!.id}+${c!.id}`, `${c!.id}+${a!.id}`].sort());
  });

  it("a bot fuses a pair only once its name is ready, or once the model gave up on it", async () => {
    // The model fails on one pair only, so it never counts as down.
    const d = content.units[3]!;
    let now = 0;
    let named = 0;
    const { rt, stored } = world(async (x, y) => {
      if (x.id === a!.id && y.id === d.id) throw new Error("503");
      return NAMES[named++]!;
    }, bot, () => 0, () => now);
    expect(fusionNameReady(rt.store, a!.id, b!.id)).toBe(false);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(fusionNameReady(rt.store, a!.id, b!.id)).toBe(true);
    rt.store.putRun({ ...stored(), line: [...stored().line, lineUnitOf(d, "u4", 3, rt.rules)], nextUid: 5 });
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    // The final failure: MODEL_FAILURES asks over MODEL_DOWN_MS.
    for (let i = 1; i < MODEL_FAILURES; i++) {
      await drainFusionNames(rt.store);
      expect(fusionNameReady(rt.store, a!.id, d.id)).toBe(false);
      now += MODEL_DOWN_MS / (MODEL_FAILURES - 1);
    }
    await drainFusionNames(rt.store);
    expect(fusionNameReady(rt.store, a!.id, d.id)).toBe(true);
    // With no model at all, every pair is ready.
    expect(fusionNameReady(new MemoryMvpStore(), a!.id, b!.id)).toBe(true);
  });

  it("failed asks in a row over MODEL_DOWN_MS put the model down: no bot waits, until it answers again", async () => {
    let down = true;
    let now = 0;
    const { rt, stored } = world(async () => {
      if (down) throw new Error("connection refused");
      return "Grimward";
    }, bot, () => 0, () => now);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // 6 pairs, each fails once
    await drainFusionNames(rt.store);
    // Six failures in no time: a model still loading, not down. Bots wait.
    expect(fusionNameReady(rt.store, a!.id, b!.id)).toBe(false);
    now += MODEL_DOWN_MS;
    await drainFusionNames(rt.store);
    expect(fusionNameReady(rt.store, a!.id, b!.id)).toBe(true);
    // Back up: one answer and the rest wait for theirs again.
    down = false;
    const d = content.units[3]!;
    rt.store.putRun({ ...stored(), line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(d, "u4", 3, rt.rules)], nextUid: 5 });
    const waited = awaitFusionName(rt.store, a!.id, d.id);
    expect(fusionNameReady(rt.store, a!.id, d.id)).toBe(true); // still down until an answer
    await drainFusionNames(rt.store); // no probe due yet
    expect(rt.peekFusionName(a!, d, bot).name).not.toBe("Grimward");
    now += MODEL_PROBE_MS;
    await drainFusionNames(rt.store);
    await waited;
    expect(rt.peekFusionName(a!, d, bot).name).toBe("Grimward");
    // Up again: a new pair is waited for.
    expect(fusionNameReady(rt.store, b!.id, d.id)).toBe(false);
  });

  it("a bot waiting for a pair has it asked next and wakes when it is named, or when the model gives up", async () => {
    const asked: string[] = [];
    let failOn = "";
    let now = 0;
    const { rt, stored } = world(async (x, y) => {
      asked.push(`${x.id}+${y.id}`);
      if (`${x.id}+${y.id}` === failOn) throw new Error("timeout");
      return NAMES[asked.length - 1]!;
    }, bot, () => 0, () => now);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // 6 pairs queued ahead
    let woke = false;
    const waited = awaitFusionName(rt.store, c!.id, b!.id).then(() => (woke = true));
    await drainFusionNames(rt.store);
    expect(asked[0]).toBe(`${c!.id}+${b!.id}`);
    await waited;
    expect(woke).toBe(true);
    expect(fusionNameReady(rt.store, c!.id, b!.id)).toBe(true);
    // A pair the model keeps failing on: the bot wakes after MODEL_FAILURES asks over MODEL_DOWN_MS.
    const d = content.units[3]!;
    failOn = `${d.id}+${a!.id}`;
    let gaveUp = false;
    const late = awaitFusionName(rt.store, d.id, a!.id).then(() => (gaveUp = true));
    for (let i = 1; i < MODEL_FAILURES; i++) {
      await drainFusionNames(rt.store);
      expect(gaveUp).toBe(false);
      now += MODEL_DOWN_MS / (MODEL_FAILURES - 1);
    }
    await drainFusionNames(rt.store);
    await late;
    expect(gaveUp).toBe(true);
  });

  /** Drives `now` a second at a time, draining the queue each step, until `done()` or `until` ms. */
  async function tick(clock: { now: number }, done: () => boolean, until: number, each?: () => void) {
    for (; !done() && clock.now <= until; clock.now += 1000) {
      each?.();
      await drainFusionNames(store!);
      await new Promise((r) => setImmediate(r));
    }
  }
  let store: MvpStore | undefined;

  it("a model still loading (503s for its first 20 s) is waited for: the champion's fused names are the model's", async () => {
    for (let seed = 1; seed <= 6; seed++) {
      let s = seed;
      const clock = { now: 0 };
      const calls: number[] = [];
      let named = 0;
      const rt = mvpRuntime({ content, seed: () => (s = (s * 1103515245 + 12345) >>> 0) });
      const naming = fusionNaming(rt, {
        model: async () => {
          calls.push(clock.now);
          if (clock.now < 20_000) throw new Error("namer answered 503");
          return NAMES[named++] ?? null;
        },
        clock: () => clock.now,
      });
      rt.hooks[0] = naming.hooks;
      rt.nameFusion = naming.nameFusion;
      rt.peekFusionName = naming.peek;
      store = rt.store;
      let champ: Champion | undefined | null = null;
      const seeding = seedChampion(rt).then((c) => (champ = c));
      await tick(clock, () => champ !== null, 10 * 60_000);
      await seeding;
      const fused = champ!.line.filter((u) => u.fusion);
      if (fused.length === 0) continue;
      expect(calls.filter((t) => t < 20_000).length).toBeGreaterThanOrEqual(fused.length);
      expect(clock.now).toBeLessThan(MODEL_DOWN_MS);
      for (const u of fused) {
        expect(NAMES).toContain(u.name);
        expect(rt.store.fusion(u.fusion!.first, u.fusion!.second)).toMatchObject({ name: u.name, nameSource: "model", discoveredBy: null });
      }
      return;
    }
    throw new Error("no seeded champion held a fusion");
  }, 60_000);

  it("a model that is really down: bots stop waiting after about a minute, then it is asked about once a minute", async () => {
    const clock = { now: 0 };
    const calls: number[] = [];
    const w = world(async () => {
      calls.push(clock.now);
      throw new Error("connection refused");
    }, bot, undefined, () => clock.now);
    store = w.rt.store;
    decide(w.rt, w.stored(), { kind: "reorder", from: 0, to: 0 }); // 6 pairs
    let woke = -1;
    void awaitFusionName(store, a!.id, b!.id).then(() => (woke = clock.now));
    // A bot wants a new pair every 10 s.
    const units = content.units;
    let k = 3;
    const end = 15 * 60_000;
    await tick(clock, () => false, end, () => {
      if (clock.now % 10_000 === 0 && k + 1 < units.length) void awaitFusionName(store!, units[k]!.id, units[++k]!.id);
    });
    // Bounded: the model counts as down after MODEL_FAILURES failures over MODEL_DOWN_MS.
    expect(woke).toBeGreaterThanOrEqual(MODEL_DOWN_MS);
    expect(woke).toBeLessThanOrEqual(MODEL_DOWN_MS + 30_000);
    // Then one probe a minute, however many new pairs bots want.
    const probes = calls.filter((t) => t > woke).length;
    const minutes = (end - woke) / MODEL_PROBE_MS;
    expect(probes).toBeGreaterThanOrEqual(Math.floor(minutes) - 1);
    expect(probes).toBeLessThanOrEqual(Math.ceil(minutes) + 1);
    // While down a new pair is ready at once: bots fuse it with the portmanteau.
    expect(fusionNameReady(store, units[k]!.id, units[1]!.id)).toBe(true);
  }, 60_000);

  it("asks for one word: two words pass only on the last try, three never, and a unit's name inside is asked again", async () => {
    const asked: string[] = [];
    const answers: Record<string, string[]> = {
      [`${a!.id}+${b!.id}`]: ["Ash Warden", "Ember-Fang", "Cinderhowl"],
      [`${b!.id}+${a!.id}`]: ["Ash Warden", "Ember-Fang", "Grim Bloom"],
      [`${a!.id}+${c!.id}`]: ["Ash Warden", "Duskmaw Iron Fang", "Iron Fang Hollow"],
      [`${c!.id}+${a!.id}`]: [`${a!.name.replace(/\s+/g, "")}wyrm`, "Gloomtusk"],
    };
    const { rt, stored } = world(async (x, y) => {
      const key = `${x.id}+${y.id}`;
      asked.push(key);
      return answers[key]?.[asked.filter((k) => k === key).length - 1] ?? NAMES[asked.length % NAMES.length]!;
    });
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(rt.store.fusion(a!.id, b!.id)).toBeUndefined();
    expect(rt.peekFusionName(a!, b!, maks).name).toBe("Cinderhowl");
    expect(rt.peekFusionName(b!, a!, maks).name).toBe("Grim Bloom");
    expect(rt.peekFusionName(a!, c!, maks).name).toBe(portmanteau(a!.name, c!.name, content.units.map((x) => x.name)));
    expect(rt.peekFusionName(c!, a!, maks).name).toBe("Gloomtusk");
  });

  it("a blocked answer keeps the portmanteau", async () => {
    let asked = 0;
    const { rt, stored } = world(async () => (asked++, "Pikachu"));
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(asked).toBe(6 * 3); // three tries for each of the 6 ordered pairs
    decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: portmanteau(a!.name, b!.name), nameSource: "fallback" });
  });

  it("a bot's fusion is stored uncredited; the first human to fuse the pair claims it, the name stays", async () => {
    const asked: string[] = [];
    const botWorld = world(async (x, y) => (asked.push(`${x.id}+${y.id}`), NAMES[asked.length - 1]!), bot);
    decide(botWorld.rt, botWorld.stored(), { kind: "reorder", from: 0, to: 0 });
    expect(botWorld.rt.store.fusions()).toEqual([]);
    // Bots' lines are asked ahead too.
    await drainFusionNames(botWorld.rt.store);
    expect(asked).toHaveLength(6);
    decide(botWorld.rt, botWorld.stored(), { kind: "fuse", first: 0, second: 1 });
    expect(botWorld.rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: "Duskmend", discoveredBy: null, nameSource: "model" });

    const { rt } = botWorld;
    const human = startRun(rt, maks);
    rt.store.putRun({ ...human, line: [lineUnitOf(a!, "h1", 3, rt.rules), lineUnitOf(b!, "h2", 3, rt.rules)], nextUid: 3 });
    const done = decide(rt, rt.store.run(human.runId)!, { kind: "fuse", first: 0, second: 1 });
    expect(done.run.line[0]).toMatchObject({ name: "Duskmend", fusion: { discoveredBy: maks } });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: "Duskmend", discoveredBy: maks });
  });
});

describe("MVP fusion names: the HTTP model client", () => {
  let server: Server | undefined;
  afterEach(() => server?.close());

  it("asks an OpenAI-compatible chat endpoint with the few-shot turns, no emoji, and returns its reply", async () => {
    let got: { messages: { role: string; content: string }[]; max_tokens: number; temperature: number } | undefined;
    server = createServer((req, res) => {
      let body = "";
      req.on("data", (c) => (body += c));
      req.on("end", () => {
        got = JSON.parse(body);
        res.setHeader("content-type", "application/json");
        res.end(JSON.stringify({ choices: [{ message: { content: "Ironcare" } }] }));
      });
    });
    await new Promise<void>((r) => server!.listen(0, "127.0.0.1", r));
    const { port } = server.address() as AddressInfo;
    const ask = httpModelNamer(`http://127.0.0.1:${port}/v1/chat/completions`, 15_000, () => 0.99);
    expect(await ask(brawler, medic)).toBe("Ironcare");
    const emojiUnit = { ...brawler, emoji: "🌹", name: "Rose" };
    expect(await ask(emojiUnit, { ...medic, emoji: "🐀", name: "Rat" })).toBe("Ironcare");
    expect(got?.messages.map((m) => m.role)).toEqual(["system", "user", "assistant", "user", "assistant", "user", "assistant", "user", "assistant", "user"]);
    // A starting letter is hinted (against Qwen3-4B's habit of S), each example's its own.
    expect(got?.messages[1]).toEqual({ role: "user", content: "Knight merges with Wolf. Start with the letter F. Name:" });
    expect(got?.messages[2]).toEqual({ role: "assistant", content: "Fangwarden" });
    expect(got?.messages.at(-1)).toEqual({ role: "user", content: `Rose merges with Rat. Start with the letter ${NAMER_LETTERS.at(-1)}. Name:` });
    expect(got?.messages.some((m) => /\p{Extended_Pictographic}/u.test(m.content))).toBe(false);
    expect(got).toMatchObject({ max_tokens: 12, temperature: 0.8 });
  });

  it("uses no content unit's name in its examples (the model echoes example words)", () => {
    const names = new Set(mvpContent().units.flatMap((u) => u.name.toLowerCase().split(/\s+/)));
    const words = NAMER_EXAMPLES.flatMap((e) => [e.first, e.second, ...e.name.split(/\s+/)]).map((w) => w.toLowerCase());
    expect(words.filter((w) => names.has(w))).toEqual([]);
  });

  it("throws on a 5xx (mlx still loading), so the pair is asked again", async () => {
    server = createServer((_req, res) => {
      res.statusCode = 503;
      res.end("loading");
    });
    await new Promise<void>((r) => server!.listen(0, "127.0.0.1", r));
    const { port } = server.address() as AddressInfo;
    await expect(httpModelNamer(`http://127.0.0.1:${port}/v1/chat/completions`)(brawler, medic)).rejects.toThrow("503");
  });
});
