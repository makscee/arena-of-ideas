// Login flow (PRD #016 slice 3) — email → 6-digit code → first-login name,
// hung off the title screen's #title-login entry. One form, three steps, one
// state machine; every server answer lands as visible text (a rate limit, a
// wrong code, a dead network), never a bricked panel. On success the page
// reloads: boot (main.ts) reads the persisted token and wires the whole app
// remote — login is a mode switch, not a hot swap, so the store seam stays
// chosen exactly once.

import type { ArenaApi, MeInfo } from "./api.js";

type Step = "email" | "code" | "name";

export interface LoginEls {
  /** The logged-out entry on the title menu. */
  loginButton: HTMLButtonElement;
  /** The logged-in identity strip: name + logout. */
  identity: HTMLElement;
  identityName: HTMLElement;
  logoutButton: HTMLButtonElement;
  /** The step panel. */
  panel: HTMLElement;
  form: HTMLFormElement;
  blurb: HTMLElement;
  emailRow: HTMLElement;
  email: HTMLInputElement;
  codeRow: HTMLElement;
  code: HTMLInputElement;
  nameRow: HTMLElement;
  name: HTMLInputElement;
  submit: HTMLButtonElement;
  cancel: HTMLButtonElement;
  error: HTMLElement;
}

export interface LoginDeps {
  api: ArenaApi;
  /** Who is logged in at boot (null = logged out). */
  identity: MeInfo | null;
  /** The boot token, when one exists — logout and the name pick use it. */
  token: string | null;
  saveToken(token: string): void;
  clearToken(): void;
  /** Page reload — the mode switch after login/logout/name pick. */
  reload(): void;
}

export function createLogin(els: LoginEls, deps: LoginDeps): void {
  let step: Step = "email";
  // The token minted mid-flow (verify → name pick) before any reload.
  let flowToken: string | null = deps.token;
  let flowEmail = "";

  function setError(message: string | null): void {
    els.error.hidden = message === null;
    els.error.textContent = message ?? "";
  }

  function showStep(next: Step): void {
    step = next;
    setError(null);
    els.emailRow.hidden = next !== "email";
    els.codeRow.hidden = next !== "code";
    els.nameRow.hidden = next !== "name";
    els.blurb.textContent =
      next === "email"
        ? "Log in to play on the shared ladder — we'll email you a one-time code."
        : next === "code"
          ? `We sent a 6-digit code to ${flowEmail} — it lives 10 minutes.`
          : "First login: pick a display name. It's what the leaderboard shows.";
    els.submit.textContent = next === "email" ? "Send code" : next === "code" ? "Log in" : "Save name";
    (next === "email" ? els.email : next === "code" ? els.code : els.name).focus();
  }

  function openPanel(at: Step): void {
    els.panel.hidden = false;
    els.loginButton.hidden = true;
    showStep(at);
  }

  function closePanel(): void {
    els.panel.hidden = true;
    els.loginButton.hidden = deps.identity !== null;
    els.code.value = "";
  }

  /** One in-flight call at a time; the submit button is the latch. */
  async function busy<T>(work: () => Promise<T>): Promise<T> {
    els.submit.disabled = true;
    try {
      return await work();
    } finally {
      els.submit.disabled = false;
    }
  }

  async function submitEmail(): Promise<void> {
    const email = els.email.value.trim();
    if (email === "" || !email.includes("@")) {
      setError("Type the email to send the code to.");
      return;
    }
    const res = await busy(() => deps.api.startLogin(email));
    if (!res.ok) {
      setError(
        res.kind === "network"
          ? `The server didn't answer (${res.reason}) — try again in a moment.`
          : res.kind === "rejected" && res.status === 429
            ? `Too many codes requested — ${res.reason}.`
            : `That didn't work: ${res.kind === "rejected" ? res.reason : "unauthorized"}.`,
      );
      return;
    }
    flowEmail = email;
    showStep("code");
  }

  async function submitCode(): Promise<void> {
    const code = els.code.value.trim();
    if (!/^\d{6}$/.test(code)) {
      setError("The code is the 6 digits from the email.");
      return;
    }
    const res = await busy(() => deps.api.verifyLogin(flowEmail, code));
    if (!res.ok) {
      setError(
        res.kind === "network"
          ? `The server didn't answer (${res.reason}) — try again in a moment.`
          : "That code didn't verify — mistyped, expired, or used up. You can request a fresh one.",
      );
      return;
    }
    flowToken = res.value.token;
    deps.saveToken(flowToken);
    // First login has no display name yet — pick one before entering.
    const who = await busy(() => deps.api.me(flowToken!));
    if (who.ok && who.value.displayName === null) {
      showStep("name");
      return;
    }
    deps.reload();
  }

  async function submitName(): Promise<void> {
    if (flowToken === null) return; // unreachable: the name step follows a mint
    const name = els.name.value.trim();
    if (name.length < 2 || name.length > 24) {
      setError("A display name is 2–24 characters.");
      return;
    }
    const res = await busy(() => deps.api.setDisplayName(flowToken!, name));
    if (!res.ok) {
      setError(
        res.kind === "network"
          ? `The server didn't answer (${res.reason}) — try again in a moment.`
          : res.kind === "unauthorized"
            ? "The session expired mid-login — start over."
            : "Names are 2–24 letters, digits, spaces or light punctuation.",
      );
      return;
    }
    deps.reload();
  }

  els.form.addEventListener("submit", (ev) => {
    ev.preventDefault();
    void (step === "email" ? submitEmail() : step === "code" ? submitCode() : submitName());
  });
  els.cancel.addEventListener("click", closePanel);
  els.loginButton.addEventListener("click", () => openPanel("email"));
  els.logoutButton.addEventListener("click", () => {
    void (async () => {
      els.logoutButton.disabled = true;
      // Best-effort revoke — a dead server must not trap a player logged in;
      // the local token clears either way and local play returns on reload.
      if (deps.token !== null) await deps.api.logout(deps.token);
      deps.clearToken();
      deps.reload();
    })();
  });

  // Boot state: logged in shows identity; a half-finished first login (token
  // but no name yet) reopens the name pick — the leaderboard needs the name.
  if (deps.identity === null) {
    els.identity.hidden = true;
    els.loginButton.hidden = false;
    els.panel.hidden = true;
  } else {
    els.identity.hidden = false;
    els.loginButton.hidden = true;
    els.identityName.textContent = deps.identity.displayName ?? deps.identity.email;
    if (deps.identity.displayName === null) openPanel("name");
    else els.panel.hidden = true;
  }
}
