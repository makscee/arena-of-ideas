// Slice 13 (mission #574): invite links. An invite names one player; its code
// is the secret in the link (…/arena/?invite=<code>). Opening it on a device
// starts a session for that player: a fresh token the device keeps and sends
// as X-Arena-Token. The same link on a second device gives the same player.
// Invites are made on the host (./invite-cli.ts); the API only opens them.
import { createHash, randomBytes, randomUUID } from "node:crypto";
import type { PlayerRef, PlayerSession } from "../../../src/mvp/contract.js";
import type { Invite, MvpStore } from "./store.js";

export const NAME_RE = /^[\p{L}\p{N}_\- ]{1,24}$/u;

export const hashToken = (token: string) => createHash("sha256").update(token).digest("hex");

export class InviteError extends Error {}

export interface NewInvite {
  name: string;
  /** Claim this existing human player instead of making a new one. */
  playerId?: string | undefined;
  admin?: boolean;
  now: Date;
}

/** Makes one person's invite. A new name gets a new player; a name already
 * taken by a human player (or an invite) is refused unless `playerId` claims
 * that player, which is how an existing player gets a link and keeps their
 * runs, rating and fusions. A player who already has an invite gets it back. */
export function createInvite(store: MvpStore, req: NewInvite): Invite {
  const name = req.name.trim();
  if (!NAME_RE.test(name)) throw new InviteError("name: 1–24 letters, digits, spaces, _ or -");
  const all = store.invites();
  let player: PlayerRef;
  if (req.playerId) {
    const p = store.player(req.playerId);
    if (!p || p.bot) throw new InviteError(`no human player ${req.playerId}`);
    const had = all.find((i) => i.playerId === p.id);
    if (had) return had;
    if (p.name.toLowerCase() !== name.toLowerCase()) throw new InviteError(`player ${p.id} is named ${p.name}, not ${name}`);
    player = p;
  } else {
    const taken = store.playersNamed(name);
    if (taken.length) throw new InviteError(`${name} is taken by ${taken.map((p) => p.id).join(", ")}: claim one with --player <id>`);
    player = { id: randomUUID(), name, bot: false };
  }
  if (all.some((i) => i.name.toLowerCase() === name.toLowerCase())) throw new InviteError(`an invite named ${name} exists`);
  const invite: Invite = { code: randomBytes(12).toString("base64url"), name, playerId: player.id, admin: req.admin ?? false, createdAt: req.now.toISOString(), redeemedAt: null };
  if (!req.playerId) store.addPlayer(player);
  store.putInvite(invite);
  return invite;
}

/** Opens an invite link: a new session for its player, or undefined for an unknown code. */
export function redeemInvite(store: MvpStore, code: string, now: Date): PlayerSession | undefined {
  const invite = store.invite(code);
  const player = invite && store.player(invite.playerId);
  if (!invite || !player) return undefined;
  const token = randomBytes(24).toString("base64url");
  store.addSession(hashToken(token), player.id, now.toISOString());
  if (!invite.redeemedAt) store.putInvite({ ...invite, redeemedAt: now.toISOString() });
  return { player, token };
}

/** The player holding this session token. */
export function sessionPlayer(store: MvpStore, token: string): PlayerRef | undefined {
  const id = store.sessionPlayer(hashToken(token));
  return id ? store.player(id) : undefined;
}

/** May this player use the dev tools on an invite-only server? Admin invites only. */
export const isAdmin = (store: MvpStore, playerId: string | undefined) =>
  !!playerId && store.invites().some((i) => i.admin && i.playerId === playerId);
