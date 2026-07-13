import { describeStatus, type StatusRegistry } from "../src/index.js";
import { statusChipStyle } from "./status-color.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** Status chips shared by cards and inspectors without coupling either module
 * to the other. */
export function chipsHtml(
  statuses: readonly { status: string; stacks: number }[] | undefined,
  registry: StatusRegistry,
): string {
  return (statuses ?? [])
    .map((s) => {
      const def = registry[s.status];
      const title = `${s.status} ×${s.stacks}${def !== undefined ? ` — ${describeStatus(def)}` : ""}`;
      return `<span class="chip" data-status="${esc(s.status)}" style="${statusChipStyle(s.status)}" title="${esc(title)}">${esc(s.status.slice(0, 3))}${s.stacks}</span>`;
    })
    .join("");
}
