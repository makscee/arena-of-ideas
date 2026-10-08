// "discovered by" on a fused unit (mission #574, slice 10). The name and the
// credit come from the server's store (FusionDiscovery), copied onto the
// fused unit; a bot's discovery has no credit until a human fuses the pair.
import type { LineUnit } from "../../src/mvp/contract";
import { savedPlayer } from "../api";
import { t } from "../i18n";
import { h } from "./dom";

/** "discovered by you" / "discovered by @name" for a fused unit's sheet; null
 * otherwise. In a fusion preview (`preview`), a pair nobody has fused says it
 * is named when you fuse, and a pair only bots have made says so. */
export function discoveredLine(u: Partial<Pick<LineUnit, "kind" | "fusion">>, o: { preview?: boolean } = {}): HTMLElement | null {
  if (u.kind !== "fused" || !u.fusion) return null;
  const line = (text: string) => h("div", { class: "discovered", "data-testid": "discovered-by" }, text);
  if (o.preview && u.fusion.name === "") return line(t("fusion.newNamedWhenFused"));
  const by = u.fusion.discoveredBy;
  if (!by) return o.preview ? line(t("fusion.madeByBots")) : null;
  return line(by.id === savedPlayer()?.id ? t("fusion.discoveredByYou") : t("fusion.discoveredBy", { name: by.name }));
}

/** The name a preview shows for a pair nobody has fused (the server sends no
 * name until the fuse): "??? New fusion" on the sheet, `label` on a card. */
export function previewName(u: LineUnit, label = t("fusion.previewName")): LineUnit {
  return u.kind === "fused" && u.name === "" ? { ...u, name: label } : u;
}
