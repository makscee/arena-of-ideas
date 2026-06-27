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

import { statusPill, voteScore, type Idea, type VoteDir } from "../src/index.js";
import type { RemoteIdeas } from "./remote-ideas.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** How the ladder is ordered: `top` is the server's score rank (the default),
 * `new` is newest-first (submission order, highest seq first). A display-only
 * re-sort — it never refetches or re-ranks the server's truth. */
export type IdeasSort = "top" | "new";

export interface IdeasLadderOpts {
  /** The signed-in player's id (drives the per-row "you voted" toggle), or null. */
  userId: string | null;
  /** Top (score rank, as the server returned) or New (newest seq first). */
  mode: IdeasSort;
}

/** A short, stable author label off the author id — server ids are opaque, so a
 * long uuid is trimmed to keep the row legible (never a raw email). */
function authorLabel(authorId: string): string {
  return authorId.length > 12 ? `${authorId.slice(0, 10)}…` : authorId;
}

/** One creation-ladder row (mockup line 79): a 32px vote column — ▲ upvote, the
 * net score, ▼ downvote — then the idea text over a meta line (@author + a
 * lifecycle status pill). The arrow matching the player's current direction reads
 * pressed; a `rejected` (bounced) idea dims its text and shows the bounce reason. */
function ideaRowHtml(idea: Idea, userId: string | null): string {
  const myDir: VoteDir | null = userId !== null && userId in idea.votes ? idea.votes[userId]! : null;
  const pill = statusPill(idea.status);
  const rowCls = ["ideas-row", pill === "rejected" && "is-rejected"].filter(Boolean).join(" ");
  const arrow = (dir: VoteDir, glyph: string): string => {
    const active = myDir === dir;
    return (
      `<button type="button" class="ideas-vote-arrow ideas-vote-${dir}${active ? " ideas-voted" : ""}" ` +
      `data-vote-dir="${dir}" aria-pressed="${active}">${glyph}</button>`
    );
  };
  const bounce =
    idea.status === "bounced" && idea.bounceReason !== undefined
      ? `<div class="ideas-bounce-reason">Bounced — ${esc(idea.bounceReason)}</div>`
      : "";
  return (
    `<div class="${rowCls}" data-idea-id="${esc(idea.id)}">` +
    `<div class="ideas-vote">${arrow("up", "▲")}<span class="ideas-vote-count">${voteScore(idea.votes)}</span>${arrow("down", "▼")}</div>` +
    `<div class="ideas-body">` +
    `<div class="ideas-text">${esc(idea.text)}</div>` +
    `<div class="ideas-meta"><span class="ideas-author">@${esc(authorLabel(idea.authorId))}</span>` +
    `<span class="ideas-pill ideas-pill-${pill}">${pill}</span></div>` +
    bounce +
    `</div></div>`
  );
}

/** The creation ladder as markup — one ranked pass over the ideas. Pure
 * presentation over the data the store returned: takes the ideas + display opts,
 * returns the rows' HTML to drop into any container (so the title hub can mount
 * the same ladder later). `new` re-sorts newest-first; `top` keeps the server's
 * score rank. Empty → an empty-state line. */
export function ideasLadderHtml(ideas: readonly Idea[], opts: IdeasLadderOpts): string {
  const ordered = opts.mode === "new" ? [...ideas].sort((a, b) => b.seq - a.seq) : ideas;
  if (ordered.length === 0) {
    return `<p class="ideas-empty">${
      opts.userId !== null ? "No ideas yet — be the first to write one." : "No ideas yet."
    }</p>`;
  }
  return ordered.map((idea) => ideaRowHtml(idea, opts.userId)).join("");
}

export interface IdeasScreenEls {
  /** The submit box: a text input + its send button. Hidden behind the reveal
   * button until the player opens it (the mockup's footer CTA). */
  form: HTMLFormElement;
  text: HTMLInputElement;
  submit: HTMLButtonElement;
  /** The footer CTA that reveals the submit box; logged out it routes to login. */
  reveal: HTMLButtonElement;
  /** The Top / New ladder-order toggle (Top = score rank, New = newest first). */
  sortTop: HTMLButtonElement;
  sortNew: HTMLButtonElement;
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

  // The ladder's display order, and the last list the server handed back — a
  // Top/New toggle re-renders THIS cache (a re-sort is display-only; it never
  // refetches or re-ranks the server's truth).
  let mode: IdeasSort = "top";
  let lastIdeas: readonly Idea[] = [];

  /** Render one ranked pass of the ladder from the cached list. The rows are
   * built by `ideasLadderHtml` (the reusable, hub-droppable render); vote taps
   * are handled by one delegated listener on the list, not per-button. */
  function render(ideas: readonly Idea[]): void {
    lastIdeas = ideas;
    els.list.innerHTML = ideasLadderHtml(ideas, { userId: deps.userId, mode });
  }

  function setMode(next: IdeasSort): void {
    if (mode === next) return;
    mode = next;
    els.sortTop.classList.toggle("is-active", next === "top");
    els.sortNew.classList.toggle("is-active", next === "new");
    els.sortTop.setAttribute("aria-pressed", String(next === "top"));
    els.sortNew.setAttribute("aria-pressed", String(next === "new"));
    render(lastIdeas);
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

  // One delegated vote listener over the whole list — a tap on either arrow
  // (the only [data-vote-dir] elements) casts that direction on its row's idea.
  els.list.addEventListener("click", (ev) => {
    const btn = (ev.target as HTMLElement).closest<HTMLElement>("[data-vote-dir]");
    if (btn === null) return;
    const row = btn.closest<HTMLElement>("[data-idea-id]");
    if (row === null) return;
    void onVote(row.dataset.ideaId!, btn.dataset.voteDir as VoteDir);
  });

  // The footer CTA: logged out it's a login nudge (like every other authed
  // action on the screen); logged in it reveals the submit box and focuses it.
  els.reveal.addEventListener("click", () => {
    if (!loggedIn) {
      deps.onNeedLogin();
      return;
    }
    els.form.hidden = false;
    els.reveal.hidden = true;
    els.text.focus();
  });

  els.sortTop.addEventListener("click", () => setMode("top"));
  els.sortNew.addEventListener("click", () => setMode("new"));

  return { refresh };
}
