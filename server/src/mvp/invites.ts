// Slice 13 (mission #574): invite links. An invite names one player; its code
// is the secret in the link (…/arena/#invite=<code>). Opening it on a device
// starts a session for that player: a fresh token the device keeps and sends
// as X-Arena-Token. The same link on a second device gives the same player.
// Invites are made on the host (./invite-cli.ts); the API only opens them.
import { createHash, randomBytes, randomUUID, timingSafeEqual } from "node:crypto";
import type { JoinSession, PlayerRef, PlayerSession } from "../../../src/mvp/contract.js";
import { nameKey, type Invite, type MvpStore } from "./store.js";

export const NAME_RE = /^[\p{L}\p{N}_\- ]{1,24}$/u;

export const hashToken = (token: string) => createHash("sha256").update(token).digest("hex");

export class InviteError extends Error {}
/** A join (R4-20) refused for its name: 400 unreadable, 409 taken or a bot's. */
export class JoinRefused extends InviteError {
  constructor(readonly status: 400 | 409, message: string) {
    super(message);
  }
}

export interface NewInvite {
  name: string;
  /** Claim this existing human player instead of making a new one. */
  playerId?: string | undefined;
  /** true promotes, false demotes; unset leaves an existing invite's as is (a new one: false). */
  admin?: boolean | undefined;
  now: Date;
}

const newCode = () => randomBytes(12).toString("base64url");

/** Makes one person's invite. A new name gets a new player; a name already
 * taken by a human player (or an invite) is refused unless `playerId` claims
 * that player, which is how an existing player gets a link and keeps their
 * runs, rating and fusions. A bot's name (any `bot-…` name) is never an
 * invite's; the roster word alone ("Mira") is fine, it may be a real name. A player who already has an invite gets it back, with
 * `admin` changed when it is given. */
export function createInvite(store: MvpStore, req: NewInvite): Invite {
  const name = req.name.trim();
  if (!NAME_RE.test(name)) throw new InviteError("name: 1–24 letters, digits, spaces, _ or -");
  const key = nameKey(name);
  const all = store.invites();
  const had = req.playerId ? all.find((i) => i.playerId === req.playerId) : undefined;
  if (had) {
    if (req.admin === undefined || req.admin === had.admin) return had;
    const next = { ...had, admin: req.admin };
    store.putInvite(next);
    return next;
  }
  if (key.startsWith("bot-") || store.playersNamed(name, { bots: true }).some((p) => p.bot))
    throw new InviteError(`${name} is a bot's name`);
  let player: PlayerRef;
  if (req.playerId) {
    const p = store.player(req.playerId);
    if (!p || p.bot) throw new InviteError(`no human player ${req.playerId}`);
    if (nameKey(p.name) !== key) throw new InviteError(`player ${p.id} is named ${p.name}, not ${name}`);
    player = p;
  } else {
    const taken = store.playersNamed(name);
    if (taken.length) throw new InviteError(`${name} is taken by ${taken.map((p) => p.id).join(", ")}: claim one with --player <id>`);
    player = { id: randomUUID(), name, bot: false };
  }
  if (all.some((i) => nameKey(i.name) === key)) throw new InviteError(`an invite named ${name} exists`);
  const invite: Invite = { code: newCode(), name, playerId: player.id, admin: req.admin ?? false, createdAt: req.now.toISOString(), redeemedAt: null };
  if (!req.playerId) store.addPlayer(player);
  store.putInvite(invite);
  return invite;
}

/** Revokes a person's link and gives them a new one: every device's session
 * ends (it lands on the invite screen) and the old link stops working. The
 * player, runs and rating stay. Undefined when no invite has this name. */
export function revokeInvite(store: MvpStore, name: string, now: Date): { invite: Invite; sessions: number } | undefined {
  const key = nameKey(name.trim());
  const old = store.invites().find((i) => nameKey(i.name) === key);
  if (!old) return undefined;
  const invite: Invite = { ...old, code: newCode(), createdAt: now.toISOString(), redeemedAt: null };
  return { invite, sessions: store.rotateInvite(old.code, invite) };
}

/** Opens an invite link: a new session for its player, or undefined for an unknown code. */
export function redeemInvite(store: MvpStore, code: string, now: Date): PlayerSession | undefined {
  const token = randomBytes(24).toString("base64url");
  const id = store.redeemInvite(code, hashToken(token), now.toISOString());
  const player = id && store.player(id);
  return player ? { player, token } : undefined;
}

/** The player holding this session token. */
export function sessionPlayer(store: MvpStore, token: string): PlayerRef | undefined {
  const id = store.sessionPlayer(hashToken(token));
  return id ? store.player(id) : undefined;
}

/** May this player use the dev tools on an invite-only server? Admin invites only. */
export const isAdmin = (store: MvpStore, playerId: string | undefined) =>
  !!playerId && store.invites().some((i) => i.admin && i.playerId === playerId);

// R4-20 (#704): the open join link, …/arena/#join=<code>. One shared code; whoever
// holds it picks a name and becomes a new player with their own invite (their
// link for another device). Made and rotated on the host (./invite-cli.ts:
// `open`, `open --rotate`); rotating stops new joins on the old link, and the
// players who joined keep their sessions and their own links.

/** The join code, made on first call; `rotate` replaces it. */
export function openJoin(store: MvpStore, rotate = false): string {
  const had = store.joinCode();
  if (had && !rotate) return had;
  const code = newCode();
  store.setJoinCode(code);
  return code;
}

/** Is `code` the join code? Compared in constant time (hashes of equal length). */
export function isJoinCode(store: MvpStore, code: string): boolean {
  const want = store.joinCode();
  if (!want || !code) return false;
  return timingSafeEqual(createHash("sha256").update(code).digest(), createHash("sha256").update(want).digest());
}

/** Joins through the open link: a new player named `name`, never admin, with
 * their own invite opened on this device. The name follows createInvite's
 * rules (no taken name, no bot's); undefined for a code that isn't the join code. */
export function joinOpen(store: MvpStore, code: string, name: string, now: Date): JoinSession | undefined {
  if (!isJoinCode(store, code)) return undefined;
  const n = name.trim();
  if (!NAME_RE.test(n)) throw new JoinRefused(400, "name: 1–24 letters, digits, spaces, _ or -");
  const key = nameKey(n);
  if (key.startsWith("bot-") || store.playersNamed(n, { bots: true }).some((p) => p.bot)) throw new JoinRefused(409, `${n} is a bot's name: pick another`);
  if (store.playersNamed(n).length || store.invites().some((i) => nameKey(i.name) === key)) throw new JoinRefused(409, `${n} is taken: pick another name`);
  const invite = createInvite(store, { name: n, admin: false, now });
  const session = redeemInvite(store, invite.code, now);
  if (!session) throw new Error(`join: the new invite for ${n} didn't open`);
  return { ...session, invite: invite.code };
}
