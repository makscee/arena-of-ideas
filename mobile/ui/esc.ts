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
