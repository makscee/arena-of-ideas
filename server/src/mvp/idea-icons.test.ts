// #1122: the fake reader's archetype icons picture the idea's subject; one
// neutral icon when nothing in the idea is a thing to picture.
import { describe, expect, it } from "vitest";
import { fakeIdeaReader, ideaIcon, NEUTRAL_ICON } from "./idea-reader.js";

const icons = async (text: string): Promise<string[]> => (await fakeIdeaReader().archetypes(text, { want: 3, units: [] } as never)).options.map((o) => o.emoji);

describe("archetype icons fit the idea (#1122)", () => {
  it("a lantern keeper gets a lantern on all three, never a shell, candle or hedgehog", async () => {
    expect(await icons("A lantern keeper who heals the ally behind it whenever an enemy dies")).toEqual(["🏮", "🏮", "🏮"]);
  });

  it("a Russian idea reads its own subject", async () => {
    expect(await icons("Старый волк, который кусает самого слабого врага")).toEqual(["🐺", "🐺", "🐺"]);
    expect(ideaIcon("Фонарщик, что светит союзникам")).toBe("🏮");
  });

  it("an idea with nothing to picture gets one neutral icon for all three", async () => {
    expect(await icons("Whenever something happens, it does a little more of it")).toEqual([NEUTRAL_ICON, NEUTRAL_ICON, NEUTRAL_ICON]);
  });

  it.each([
    ["A frost giant that slows the line", "❄️"],
    ["The old knight takes the first hit", "🛡️"],
    ["A magnet that pulls curses", "🧲"],
    ["A battle drummer", "🥁"],
    ["A clockwork owl that counts turns", "⚙️"],
  ])("%s → %s", (text, icon) => expect(ideaIcon(text)).toBe(icon));

  it("a short stem is a whole word: 'battle' is no bat, 'catch' no cat, 'магнит' no mage", () => {
    expect(ideaIcon("A unit that wins every battle")).toBe(NEUTRAL_ICON);
    expect(ideaIcon("Catch the enemy off guard")).toBe("🛡️");
    expect(ideaIcon("Магнит, что тянет проклятия")).toBe("🧲");
  });
});
