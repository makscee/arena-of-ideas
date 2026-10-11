// #929: email login. Sign up is a name and an email, then the 6-digit code
// mailed there; signing in on another device is the email, then the code. A
// player from before #929 (no email yet) adds one the same way before playing.
// The server's refusals come back in English (M4-2 note), so the ones these
// screens can meet are read here in the player's language.
import { api, ApiError } from "../api";
import { t } from "../i18n";
import { button, h, show } from "../ui/dom";

export interface EmailFlow {
  /** Runs once this device is the player (or the email is attached). */
  onDone: () => void;
}

/** A refusal of the email routes in the player's words. */
export function emailRefusal(e: unknown): string {
  if (!(e instanceof ApiError)) return e instanceof Error ? e.message : String(e);
  const m = e.message;
  const tries = /(\d+) tr(?:y|ies) left/.exec(m);
  if (tries) return t("email.wrong", { n: Number(tries[1]) });
  if (/no tries left/.test(m)) return t("email.noTries");
  if (/look like an email/.test(m)) return t("email.bad");
  if (/^name:/.test(m)) return t("email.badName");
  const taken = /^(.+) is (?:taken|a bot)/.exec(m);
  if (taken) return t("email.taken", { name: taken[1]! });
  if (/belongs to another player/.test(m)) return t("email.otherPlayer");
  if (/invite-only/.test(m)) return t("email.inviteOnly");
  if (/wait a minute/.test(m)) return t("email.wait");
  if (/new players this hour/.test(m)) return t("email.tooManyPlayers");
  if (/codes this hour/.test(m)) return t("email.tooMany");
  if (/login is off/i.test(m)) return t("email.off");
  if (e.status === 502) return t("email.notSent");
  if (e.status === 404) return t("email.expired");
  return m;
}

let busy = false;
/** One request at a time; a refusal lands in `err`. */
async function run(err: HTMLElement, fn: () => Promise<void>): Promise<void> {
  if (busy) return;
  busy = true;
  document.getElementById("app")?.classList.add("busy");
  try {
    err.textContent = "";
    await fn();
  } catch (e) {
    err.textContent = emailRefusal(e);
  } finally {
    busy = false;
    document.getElementById("app")?.classList.remove("busy");
  }
}

const errorLine = () => h("div", { class: "error", "data-testid": "error" });
const emailInput = (testid: string) => h("input", { type: "email", placeholder: t("email.placeholder"), maxlength: "254", autocomplete: "email", inputmode: "email", autocapitalize: "off", spellcheck: "false", "data-testid": testid });
const nameInput = (testid: string) => h("input", { placeholder: t("join.yourName"), maxlength: "24", autocomplete: "nickname", "data-testid": testid });
const onEnter = (input: HTMLInputElement, go: () => void) => input.addEventListener("keydown", (e) => e.key === "Enter" && go());
/** The plain check before a request: an @ with something on both sides. */
const looksLikeEmail = (s: string) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s.trim());

/** Sign up: a name and an email (`join`: the join code, "" when open to all),
 * with Sign in under it for a player who has one, and `extra` (Telegram). */
export function emailSignUp(opts: EmailFlow & { join: string; extra?: HTMLElement | null }): void {
  const name = nameInput("join-name");
  const email = emailInput("join-email");
  const err = errorLine();
  const go = () => void run(err, async () => {
    if (!name.value.trim()) throw new Error(t("email.nameNeeded"));
    if (!looksLikeEmail(email.value)) throw new Error(t("email.bad"));
    await api.emailStart(email.value.trim(), name.value.trim());
    codeStep({ ...opts, email: email.value.trim(), name: name.value.trim(), back: () => emailSignUp(opts) });
  });
  onEnter(name, () => email.focus());
  onEnter(email, go);
  show(
    h("h1", {}, t("join.title")),
    h("p", { class: "dim" }, t("email.signUpAbout")),
    name,
    email,
    button(t("join.play"), go, "primary", "join-submit"),
    button(t("email.signInOpen"), () => emailSignIn({ ...opts, back: () => emailSignUp(opts) }), "", "email-signin-open"),
    opts.extra ?? null,
    err,
  );
  name.focus();
}

/** Sign in on this device: the email, then the code. `join` rides along in
 * case the email has no player yet and the player picks a name. */
export function emailSignIn(opts: EmailFlow & { back?: () => void; join?: string }): void {
  const email = emailInput("signin-email");
  const err = errorLine();
  const go = () => void run(err, async () => {
    if (!looksLikeEmail(email.value)) throw new Error(t("email.bad"));
    await api.emailStart(email.value.trim());
    codeStep({ ...opts, email: email.value.trim(), back: () => emailSignIn(opts) });
  });
  onEnter(email, go);
  show(
    h("h1", {}, t("email.signInTitle")),
    h("p", { class: "dim" }, t("email.signInAbout")),
    email,
    button(t("email.sendCode"), go, "primary", "signin-submit"),
    opts.back ? button(t("email.back"), opts.back, "", "signin-back") : null,
    err,
  );
  email.focus();
}

/** A signed-in player without an email adds one before playing. */
export function emailAttach(opts: EmailFlow & { playerName: string }): void {
  const email = emailInput("attach-email");
  const err = errorLine();
  const go = () => void run(err, async () => {
    if (!looksLikeEmail(email.value)) throw new Error(t("email.bad"));
    await api.emailStart(email.value.trim());
    codeStep({ ...opts, email: email.value.trim(), attach: true, back: () => emailAttach(opts) });
  });
  onEnter(email, go);
  show(
    h("h1", {}, t("email.attachTitle")),
    h("p", { class: "dim", "data-testid": "email-attach" }, t("email.attachAbout", { name: opts.playerName })),
    email,
    button(t("email.sendCode"), go, "primary", "attach-submit"),
    err,
  );
  email.focus();
}

/** The code mailed to `email`: sign in, sign up (with `name`), or attach. An
 * email with no player yet asks for a name here, on the same code. */
function codeStep(o: EmailFlow & { email: string; name?: string; join?: string; attach?: boolean; back: () => void }): void {
  let name = o.name;
  const code = h("input", { placeholder: t("email.codePlaceholder"), maxlength: "6", inputmode: "numeric", autocomplete: "one-time-code", "data-testid": "email-code" });
  const nameRow = h("div", { class: "stack", hidden: "" }, h("p", { class: "dim", "data-testid": "email-needs-name" }, t("email.needsName")));
  const nameField = nameInput("email-name");
  nameRow.append(nameField);
  const note = h("p", { class: "dim small", "data-testid": "email-note" });
  const err = errorLine();
  const go = () => void run(err, async () => {
    note.textContent = "";
    if (!nameRow.hidden) {
      if (!nameField.value.trim()) throw new Error(t("email.nameNeeded"));
      name = nameField.value.trim();
    }
    if (o.attach) {
      await api.emailAttach(o.email, code.value.trim());
      return o.onDone();
    }
    let r;
    try {
      r = await api.emailVerify(o.email, code.value.trim(), name, o.join);
    } catch (e) {
      // A taken or unreadable name: pick another here, on the same code.
      if (e instanceof ApiError && (e.status === 409 || /^name:/.test(e.message))) {
        nameRow.hidden = false;
        nameField.value = name ?? "";
        nameField.focus();
      }
      throw e;
    }
    if (r.status === "done") return o.onDone();
    nameRow.hidden = false;
    nameField.focus();
  });
  const resend = () => void run(err, async () => {
    await api.emailStart(o.email, name);
    note.textContent = t("email.resent");
  });
  onEnter(code, go);
  onEnter(nameField, go);
  const dev = h("div", {});
  show(
    h("h1", {}, t("email.codeTitle")),
    h("p", { "data-testid": "email-sent" }, t("email.sent", { email: o.email })),
    code,
    nameRow,
    button(t("email.continue"), go, "primary", "email-code-submit"),
    h("div", { class: "row" }, button(t("email.resend"), resend, "small grow", "email-resend"), button(t("email.back"), o.back, "small grow", "email-back")),
    note,
    dev,
    err,
  );
  code.focus();
  // The dev server's fake mailer: a button fills the code in.
  void api.health().then((hl) => {
    if (hl.mail === "fake")
      dev.append(button(t("email.devCode"), () => void api.devMailCode(o.email).then((c) => (code.value = c), (e) => (err.textContent = emailRefusal(e))), "small", "email-dev-code"));
  }, () => {});
}
