// Ideas screen (PRD #076 slice 3, directional votes in #082) — the player-facing
// surface over the ideas table. The ranked list, a free-text submit box, and one
// DIRECTIONAL, switch-only vote per idea (an up/down arrow pair, never a remove),
// all backed by the RemoteIdeas seam (slice 2: async list/submit/vote over the
// arena server). Same shape as title-screen.ts / ladder-view.ts:
// takes its DOM in `els`, its behaviour in `deps`, and re-pulls the table on
// every refresh() — main.ts calls refresh() each time the screen shows.
//
// Votes are a PRIORITY QUEUE, never an entry gate (src/ideas.ts): voting only
// reorders the list, it admits/rejects nothing. The screen reflects exactly
// what the server returns — list() is the source of truth for ranking, vote()
// returns the post-toggle idea, so the UI never guesses a count.
//
// Logged-out players SEE the ranked table (list is public) but cannot
// submit/vote: those entries route to login (deps.onNeedLogin), the same way
// every other authed action on the title prompts a session — a logged-out tap
// is a login nudge, never a silent error.

import { voteScore, type Idea, type VoteDir } from "../src/index.js";
import type { RemoteIdeas } from "./remote-ideas.js";

export interface IdeasScreenEls {
  /** The submit box: a text input + its send button. */
  form: HTMLFormElement;
  text: HTMLInputElement;
  submit: HTMLButtonElement;
  /** Inline status line under the submit box — errors and confirmations. */
  status: HTMLElement;
  /** The ranked list mount — refresh() fills it with one row per idea. */
  list: HTMLElement;
  /** Shown above the list while logged out: read-only, log in to take part. */
  loginNote: HTMLElement;
  /** The per-player vote-currency counter — how many ideas you've voted on.
   * Shown only when logged in; refresh() re-pulls it after every cast. */
  currency: HTMLElement;
}

export interface IdeasScreenDeps {
  /** The server backing — present only when logged in. Logged out it's null:
   * the screen still reads the public table (through a tokenless `list`), but
   * submit/vote route to login instead of calling the server. */
  ideas: RemoteIdeas;
  /** The signed-in player's id (matches the playerIds in each idea's `votes`),
   * or null when logged out — drives the per-row "you voted" toggle state. */
  userId: string | null;
  /** Route an unauthenticated submit/vote attempt to the login flow — mirrors
   * how the title's other authed actions prompt a session. */
  onNeedLogin(): void;
}

export interface IdeasScreen {
  /** Re-pull list() and re-render the ranked table. Called every time the
   * screen shows, so the list is never stale across navigations. */
  refresh(): Promise<void>;
}

export function createIdeasScreen(els: IdeasScreenEls, deps: IdeasScreenDeps): IdeasScreen {
  const loggedIn = deps.userId !== null;

  els.loginNote.hidden = loggedIn;
  // Logged out, the submit box is a login nudge: it stays visible (so the
  // affordance reads) but a tap routes to login rather than the server.
  els.text.disabled = !loggedIn;

  function setStatus(message: string | null, kind: "error" | "ok" = "error"): void {
    els.status.hidden = message === null;
    els.status.textContent = message ?? "";
    els.status.classList.toggle("ideas-status-bad", kind === "error");
    els.status.classList.toggle("ideas-status-ok", kind === "ok");
  }

  /** Render one ranked pass of the table. Each row carries an up/down arrow pair
   * — directional, SWITCH-ONLY: there is no "remove" affordance (a vote, once
   * cast, only flips up↔down). The arrow matching the player's current direction
   * reads pressed (server truth: the player's id maps to "up"/"down" in `votes`),
   * and the count between them is the idea's net score (the rank metric). */
  function render(ideas: readonly Idea[]): void {
    els.list.textContent = "";
    if (ideas.length === 0) {
      const empty = document.createElement("p");
      empty.className = "ideas-empty";
      empty.textContent = loggedIn
        ? "No ideas yet — be the first to write one."
        : "No ideas yet.";
      els.list.append(empty);
      return;
    }
    for (const idea of ideas) {
      const row = document.createElement("div");
      row.className = "ideas-row";
      row.dataset.ideaId = idea.id;

      const myDir: VoteDir | null = deps.userId !== null && deps.userId in idea.votes ? idea.votes[deps.userId]! : null;

      const votes = document.createElement("div");
      votes.className = "ideas-vote";
      votes.dataset.ideaId = idea.id;
      votes.append(
        arrow(idea.id, "up", "▲", myDir),
        countEl(voteScore(idea.votes)),
        arrow(idea.id, "down", "▼", myDir),
      );

      const text = document.createElement("span");
      text.className = "ideas-text";
      text.textContent = idea.text;

      row.append(votes, text);
      els.list.append(row);
    }
  }

  /** One directional arrow — its own ≥44px tap target. Reads pressed when it is
   * the player's current direction; a tap casts that direction (switch-only). */
  function arrow(ideaId: string, dir: VoteDir, glyph: string, myDir: VoteDir | null): HTMLButtonElement {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `ideas-vote-arrow ideas-vote-${dir}`;
    const active = myDir === dir;
    btn.classList.toggle("ideas-voted", active);
    btn.setAttribute("aria-pressed", String(active));
    btn.dataset.dir = dir;
    btn.title = loggedIn ? (dir === "up" ? "Vote up" : "Vote down") : "Log in to vote";
    btn.textContent = glyph;
    btn.addEventListener("click", () => void onVote(ideaId, dir));
    return btn;
  }

  function countEl(score: number): HTMLSpanElement {
    const span = document.createElement("span");
    span.className = "ideas-vote-count";
    span.textContent = String(score);
    return span;
  }

  /** Re-pull and render the per-player currency — the count of ideas this player
   * has voted on. Logged out there is no per-player footprint, so the counter is
   * hidden. Derived server-side, no stored counter. */
  async function refreshCurrency(): Promise<void> {
    if (!loggedIn) {
      els.currency.hidden = true;
      return;
    }
    const res = await deps.ideas.currency();
    if (!res.ok) {
      // A currency read failing is not worth a player-facing error — the table
      // still works; just leave the counter hidden until the next refresh.
      els.currency.hidden = true;
      return;
    }
    const n = res.value;
    els.currency.hidden = false;
    els.currency.textContent = `You've voted on ${n} idea${n === 1 ? "" : "s"}.`;
  }

  async function refresh(): Promise<void> {
    await refreshCurrency();
    const res = await deps.ideas.list();
    if (!res.ok) {
      els.list.textContent = "";
      const err = document.createElement("p");
      err.className = "ideas-empty";
      err.textContent = `Couldn't load the ideas — ${res.reason}.`;
      els.list.append(err);
      return;
    }
    render(res.value);
  }

  // Re-entrancy latch. onSubmit is async (a server round-trip plus a refresh),
  // but Enter/click can fire a SECOND submit before the first resolves — and a
  // submit dispatched mid-flight reads the input AFTER the first submit cleared
  // it, so it carries an empty string: a phantom "" submit between two real
  // ones. Disabling the button isn't enough (Enter bypasses it, and the clear
  // happens inside the await window), so we gate on this flag: a submit while
  // one is in flight is dropped, not queued with a stale/empty value. */
  let submitting = false;

  async function onSubmit(): Promise<void> {
    if (!loggedIn) {
      deps.onNeedLogin();
      return;
    }
    if (submitting) return; // a submit is already in flight — never double-fire
    // Snapshot the text BEFORE any await: the field is cleared on success, so
    // reading it post-await would see "" (or the next idea the user typed).
    const text = els.text.value.trim();
    if (text === "") {
      setStatus("Type an idea before sending it.");
      return;
    }
    submitting = true;
    els.submit.disabled = true;
    try {
      const res = await deps.ideas.submit(text);
      if (!res.ok) {
        setStatus(res.reason);
        return;
      }
      // Clear only if the field still holds the text we just submitted — a
      // racing keystroke between submit() resolving and here must not be eaten.
      if (els.text.value.trim() === text) els.text.value = "";
      setStatus("Idea added.", "ok");
      await refresh(); // re-pull so the new idea lands in its ranked place
    } finally {
      submitting = false;
      els.submit.disabled = false;
    }
  }

  async function onVote(ideaId: string, dir: VoteDir): Promise<void> {
    if (!loggedIn) {
      deps.onNeedLogin();
      return;
    }
    const res = await deps.ideas.vote(ideaId, dir);
    if (!res.ok) {
      setStatus(res.reason);
      return;
    }
    setStatus(null);
    // A vote moves rank — re-pull the whole table so the row lands in its new
    // position (the server already applied the cast; we render its order).
    await refresh();
  }

  els.form.addEventListener("submit", (ev) => {
    ev.preventDefault();
    void onSubmit();
  });

  return { refresh };
}
