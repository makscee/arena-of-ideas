// Shared helpers for the position words (§3.1).

import type { WordCtx, WordUnit } from "./types.js";

/** The holder's slot in its living line, or -1 once it has left it. */
export function slotOf(ctx: WordCtx, holder: WordUnit): number {
  return ctx.line(holder.side).findIndex((u) => u.id === holder.id);
}
