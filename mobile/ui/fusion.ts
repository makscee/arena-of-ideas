// "discovered by" on a fused unit (mission #574, slice 10). The name and the
// credit come from the server's store (FusionDiscovery), copied onto the
// fused unit; a bot's discovery has no credit until a human fuses the pair.
import type { LineUnit } from "../../src/mvp/contract";
import { savedPlayer } from "../api";
import { h } from "./dom";

/** "discovered by you" / "discovered by @name" for a fused unit; null
 * otherwise. `short` is the card's form, "by you", in the rates slot: it
 * wraps between "by" and the name and is never cut off (style.css). */
export function discoveredLine(u: Partial<Pick<LineUnit, "kind" | "fusion">>, short = false): HTMLElement | null {
  const by = u.kind === "fused" ? u.fusion?.discoveredBy : undefined;
  if (!by) return null;
  const who = by.id === savedPlayer()?.id ? "you" : `@${by.name}`;
  if (short) return h("div", { class: "rates discovered", "data-testid": "discovered-by", title: `discovered by ${who}` }, `by ${who}`);
  return h("div", { class: "discovered", "data-testid": "discovered-by" }, `discovered by ${who}`);
}
