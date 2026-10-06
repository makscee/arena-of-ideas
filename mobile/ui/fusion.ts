// "discovered by" on a fused unit (mission #574, slice 10). The name and the
// credit come from the server's store (FusionDiscovery), copied onto the
// fused unit; a bot's discovery has no credit until a human fuses the pair.
import type { LineUnit } from "../../src/mvp/contract";
import { savedPlayer } from "../api";
import { h } from "./dom";

/** "discovered by you" / "discovered by @name" for a fused unit's sheet; null otherwise. */
export function discoveredLine(u: Partial<Pick<LineUnit, "kind" | "fusion">>): HTMLElement | null {
  const by = u.kind === "fused" ? u.fusion?.discoveredBy : undefined;
  if (!by) return null;
  const who = by.id === savedPlayer()?.id ? "you" : `@${by.name}`;
  return h("div", { class: "discovered", "data-testid": "discovered-by" }, `discovered by ${who}`);
}
