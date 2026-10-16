// Reports (#1271, mission #1214): a player flags another player's name or a
// player-made unit, the way a portal's QA asks a player to be able to. A
// report is only stored: nothing is hidden or sent (moderation is Maks's call),
// and `npm run mvp:reports` lists the open ones.
import { randomUUID } from "node:crypto";
import type { PlayerRef } from "../../../src/mvp/contract.js";
import type { MvpRuntime } from "./runtime.js";
import type { Report } from "./store.js";

/** Reports one player may file in a rolling day. */
export const REPORTS_PER_DAY = 10;
export const REASON_MAX = 200;

export class ReportRefused extends Error {
  constructor(readonly status: 400 | 404 | 429, message: string) {
    super(message);
  }
}

/** Files `by`'s report of `body`'s target. Reporting the same target twice is quietly the same report. */
export function fileReport(rt: MvpRuntime, by: PlayerRef, body: { kind?: unknown; id?: unknown; reason?: unknown } | null): Report {
  const kind = body?.kind;
  const id = typeof body?.id === "string" ? body.id.trim() : "";
  if (kind !== "name" && kind !== "unit") throw new ReportRefused(400, "kind: name or unit");
  if (!id || id.length > 100) throw new ReportRefused(400, "id: the player's or the unit's id");
  if (body?.reason !== undefined && typeof body.reason !== "string") throw new ReportRefused(400, "reason: text");
  const reason = typeof body?.reason === "string" ? body.reason.replace(/\s+/g, " ").trim().slice(0, REASON_MAX) : "";
  // Only what players write can be reported: a human's name, a unit made from a player's idea.
  if (kind === "name") {
    const p = rt.store.player(id);
    if (!p || p.bot) throw new ReportRefused(404, "no such player");
  } else if (!rt.store.unit(id)?.authorId) throw new ReportRefused(404, "no such player-made unit");
  // A repeat of an earlier report costs nothing and adds nothing.
  const known = rt.store.reports().find((r) => r.reporterId === by.id && r.kind === kind && r.targetId === id);
  if (known) return known;
  const now = rt.now();
  const since = new Date(now.getTime() - 86_400_000).toISOString();
  if (rt.store.reportsBy(by.id, since) >= REPORTS_PER_DAY) throw new ReportRefused(429, "Too many reports today. Try again tomorrow.");
  const report: Report = { id: randomUUID(), reporterId: by.id, kind, targetId: id, ...(reason ? { reason } : {}), at: now.toISOString(), status: "open" };
  rt.store.addReport(report);
  return report;
}
