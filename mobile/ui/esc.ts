// What one Esc does (round 3, note 9): pure, so the order is tested
// (esc.test.ts); ui/dom.ts's one keydown listener acts on it.

/** Top-most thing first: the term popover, then the top overlay, then a
 * focused text field (a search with text: the browser clears it; the next
 * Esc blurs it), then the screen's own step (its onKeys: back, or the run
 * menu). */
export type EscStep = "popover" | "overlay" | "native" | "blur" | "screen";

export function escStep(s: { popover: boolean; overlays: number; field: "none" | "clearable" | "plain" }): EscStep {
  if (s.popover) return "popover";
  if (s.overlays > 0) return "overlay";
  if (s.field === "clearable") return "native";
  if (s.field === "plain") return "blur";
  return "screen";
}

/** The screens where Back is an Esc and not a step to the screen before: a
 * battle (Back skips or leaves it). Every other screen change is a history
 * entry of its own (ui/dom.ts), so Back there goes to the previous screen. */
const BACK_SCREENS = new Set(["battle"]);

/** Whether Back must close something first right now (ui/dom.ts keeps one
 * guard entry for it): a sheet is open, or a battle is playing. */
export function backStays(s: { overlays: number; screen: string }): boolean {
  return s.overlays > 0 || BACK_SCREENS.has(s.screen);
}
