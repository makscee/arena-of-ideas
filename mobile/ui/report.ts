// Report (#1271, mission #1214): a small "Report" on a player's name and on a
// player-made unit's sheet, the way a portal's QA asks a player to be able to.
// It opens a short sheet (an optional reason), posts the report and thanks the
// player. Nothing is hidden by it. The sender is set once by main.ts (this
// file stays out of api.ts's import cycle with the cards).
import { t } from "../i18n";
import { button, h, overlay } from "./dom";

export type ReportTarget = { kind: "name" | "unit"; id: string; reason?: string };
let send: (r: ReportTarget) => Promise<unknown> = () => Promise.reject(new Error("no reporter"));
let mineId: () => string = () => "";
export function setReporter(sender: (r: ReportTarget) => Promise<unknown>, myId: () => string): void {
  send = sender;
  mineId = myId;
}

/** Whether this player's own name or unit is `id`'s: nothing to report. */
export const isMine = (id: string | undefined): boolean => !!id && id === mineId();

/** The sheet: what is reported, an optional reason, Send and Cancel; "Thanks" after. */
export function openReport(kind: "name" | "unit", id: string, label: string): void {
  const reason = h("input", { type: "text", maxlength: "200", placeholder: t("report.reason"), "data-testid": "report-reason", "aria-label": t("report.reason") }) as HTMLInputElement;
  const msg = h("div", { class: "dim small", "data-testid": "report-msg" });
  const sendBtn = button(t("report.send"), () => void go(), "primary", "report-send");
  const cancel = button(t("report.cancel"), () => close(), "", "report-cancel");
  const body = h("div", { class: "stack report-sheet", "data-testid": "report-sheet" }, h("h2", {}, t(kind === "name" ? "report.titleName" : "report.titleUnit")), h("div", { class: "report-about" }, label), reason, msg);
  const actions = h("div", { class: "row sheet-actions" }, cancel, sendBtn);
  const close = overlay(body, actions);
  async function go(): Promise<void> {
    sendBtn.disabled = true;
    msg.textContent = "";
    try {
      await send({ kind, id, ...(reason.value.trim() ? { reason: reason.value.trim() } : {}) });
      body.replaceChildren(h("h2", {}, t("report.thanks")));
      actions.replaceChildren(button(t("report.close"), () => close(), "primary", "report-close"));
      actions.querySelector<HTMLElement>("button")?.focus();
    } catch (e) {
      msg.textContent = t("report.failed", { why: e instanceof Error ? e.message : String(e) });
      sendBtn.disabled = false;
    }
  }
}

/** A small "Report" button for a player's name (kind "name", the player's id) or a player-made unit. */
export function reportLink(kind: "name" | "unit", id: string, label: string): HTMLElement {
  const b = button(t("report.button"), () => openReport(kind, id, label), "link small report-link", `report-${kind}`);
  b.setAttribute("aria-label", `${t("report.button")}: ${label}`);
  return b;
}
