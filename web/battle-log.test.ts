import { describe, expect, test } from "vitest";
import { boardFirstFixture } from "./battle-fixtures.js";
import { transcriptHtml } from "./battle-log.js";
import { displayNames } from "../src/index.js";

describe("full battle transcript", () => {
  test("renders every event once with synchronized ids and structured cause ids", () => {
    const { log } = boardFirstFixture("board-first-strike-status-chain");
    const html = transcriptHtml(log, 7, displayNames(log));
    expect((html.match(/data-log-event=/g) ?? []).length).toBe(log.length);
    expect((html.match(/is-current/g) ?? []).length).toBe(1);
    expect(html).toContain('data-log-event="7"');
    expect(html).toMatch(/data-caused-by="[0-9]+"/);
    expect(html).toContain("StatusApplied");
    expect(html).not.toContain("data-card-entity");
  });
});
