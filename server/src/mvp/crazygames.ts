// #1218 (mission #1214): players from CrazyGames. A visitor with no token
// becomes a guest at once: a new player with a generated name and a session,
// no form. A CrazyGames login sends the SDK's user token; it is a JWT signed
// RS256 with CrazyGames' public key (https://docs.crazygames.com/sdk/user/),
// verified here, and its `userId` (never the username) finds or makes the
// player. A guest who logs in keeps their player: it gets the link.
import { createPublicKey, randomBytes, randomInt, randomUUID, verify, type KeyObject } from "node:crypto";
import type { PlayerRef, PlayerSession } from "../../../src/mvp/contract.js";
import { hashToken, isCrudePlayerName, NAME_RE } from "./invites.js";
import { nameKey, type MvpStore } from "./store.js";

/** New guest and CrazyGames players an hour: per address (ARENA_CLIENT_IP_HEADER's,
 * else the nearest proxy's X-Forwarded-For entry) and for the whole server.
 * The open join's JOINS_PER_HOUR (./app.ts) stays apart. */
export const GUEST_JOINS = { perAddress: 5, perHour: 300 };

const HOUR = 3_600_000;
const inHour = (ts: readonly number[], now: number) => ts.filter((t) => now - t < HOUR);

/** The hourly counts of new guest and CrazyGames players, in memory. An
 * address with no join in the last hour is forgotten. */
export function guestLimiter(limits: { perAddress: number; perHour: number }) {
  const byAddress = new Map<string, number[]>();
  let all: number[] = [];
  return {
    admit(address: string, now: number): boolean {
      for (const [a, ts] of byAddress) {
        const kept = inHour(ts, now);
        if (kept.length) byAddress.set(a, kept);
        else byAddress.delete(a);
      }
      all = inHour(all, now);
      const mine = byAddress.get(address) ?? [];
      if (mine.length >= limits.perAddress || all.length >= limits.perHour) return false;
      byAddress.set(address, [...mine, now]);
      all.push(now);
      return true;
    },
    /** Addresses counted now (tests). */
    addresses: () => byAddress.size,
  };
}

export const CG_PUBLIC_KEY_URL = "https://sdk.crazygames.com/publicKey.json";

/** A CrazyGames login refused (401): a token that doesn't verify. */
export class CrazyGamesRefused extends Error {
  constructor(readonly status: 401, message: string) {
    super(message);
  }
}

export interface CgClaims {
  userId: string;
  username: string;
}

/** CrazyGames' public key (PEM). `fresh` skips any cache: the key may have rotated. */
export type CgKeySource = (fresh: boolean) => Promise<string>;

/** The public key from CrazyGames, cached for an hour. A failed verify asks
 * again, but at most once a minute: bad tokens can't make every request a fetch. */
export function cgKeyFromWeb(url = CG_PUBLIC_KEY_URL, fetcher: typeof fetch = fetch, clock: () => number = Date.now): CgKeySource {
  let cached: { pem: string; at: number } | undefined;
  return async (fresh) => {
    const age = cached ? clock() - cached.at : Infinity;
    if (cached && age < (fresh ? 60_000 : HOUR)) return cached.pem;
    const res = await fetcher(url, { signal: AbortSignal.timeout(5_000) });
    if (!res.ok) throw new Error(`CrazyGames public key: HTTP ${res.status}`);
    const pem = ((await res.json()) as { publicKey?: unknown }).publicKey;
    if (typeof pem !== "string" || !pem.includes("PUBLIC KEY")) throw new Error("CrazyGames public key: no publicKey");
    cached = { pem, at: clock() };
    return pem;
  };
}

const b64json = (part: string): Record<string, unknown> | undefined => {
  try {
    const v = JSON.parse(Buffer.from(part, "base64url").toString("utf8")) as unknown;
    return v && typeof v === "object" && !Array.isArray(v) ? (v as Record<string, unknown>) : undefined;
  } catch {
    return undefined;
  }
};

/** The claims of a token signed RS256 by `key`, unexpired at `now` and, when
 * `gameId` is given, of that game; throws CrazyGamesRefused(401) otherwise. */
export function verifyCgJwt(token: string, key: KeyObject | string, now: Date, gameId?: string): CgClaims {
  const parts = token.split(".");
  if (parts.length !== 3) throw new CrazyGamesRefused(401, "CrazyGames login: not a token");
  const [head, body, sig] = parts as [string, string, string];
  const header = b64json(head);
  if (header?.alg !== "RS256") throw new CrazyGamesRefused(401, "CrazyGames login: not an RS256 token");
  const ok = verify("RSA-SHA256", Buffer.from(`${head}.${body}`), typeof key === "string" ? createPublicKey(key) : key, Buffer.from(sig, "base64url"));
  if (!ok) throw new CrazyGamesRefused(401, "CrazyGames login: bad signature");
  const claims = b64json(body);
  if (!claims) throw new CrazyGamesRefused(401, "CrazyGames login: unreadable token");
  const sec = now.getTime() / 1000;
  if (typeof claims.exp !== "number" || claims.exp <= sec) throw new CrazyGamesRefused(401, "CrazyGames login: token expired");
  if (typeof claims.iat === "number" && claims.iat > sec + 300) throw new CrazyGamesRefused(401, "CrazyGames login: token from the future");
  const userId = typeof claims.userId === "string" ? claims.userId.trim() : "";
  if (!userId) throw new CrazyGamesRefused(401, "CrazyGames login: no userId");
  // The docs don't say if gameId is a number or a string: compare as text.
  const tokenGame = typeof claims.gameId === "string" || typeof claims.gameId === "number" ? String(claims.gameId) : undefined;
  if (gameId !== undefined && tokenGame !== gameId) throw new CrazyGamesRefused(401, "CrazyGames login: a token of another game");
  return { userId, username: typeof claims.username === "string" ? claims.username : "" };
}

/** Verifies with the cached key, then once more with a fresh one (a rotated key). */
export async function verifyCgToken(token: string, keys: CgKeySource, now: Date, gameId?: string): Promise<CgClaims> {
  try {
    return verifyCgJwt(token, await keys(false), now, gameId);
  } catch (err) {
    if (!(err instanceof CrazyGamesRefused) || !err.message.endsWith("bad signature")) throw err;
    return verifyCgJwt(token, await keys(true), now, gameId);
  }
}

// Guest names: word + word + number, like "Amber Fox 417".
const FIRST = ["Amber", "Ash", "Brave", "Bright", "Calm", "Clever", "Copper", "Crimson", "Dawn", "Dusk", "Ember", "Frost", "Gold", "Iron", "Jade", "Lucky", "Moon", "Quiet", "Rapid", "Silver", "Storm", "Sun", "Swift", "Wild"];
const SECOND = ["Badger", "Bear", "Crane", "Drake", "Falcon", "Fox", "Hare", "Hawk", "Heron", "Lynx", "Moth", "Otter", "Owl", "Raven", "Seal", "Stag", "Tiger", "Viper", "Wolf", "Wren"];

const pick = <T>(xs: readonly T[], rnd: (n: number) => number) => xs[rnd(xs.length)]!;

/** A name no player or invite has, that isn't crude or a bot's: `wanted`
 * (a CrazyGames username, cut to fit) or a generated guest name, with a
 * number added when it is taken. */
export function freeName(store: MvpStore, wanted: string | undefined, rnd: (n: number) => number = randomInt): string {
  const taken = (n: string) => store.playersNamed(n, { bots: true }).length > 0 || store.invites().some((i) => nameKey(i.name) === nameKey(n));
  const ok = (n: string) => NAME_RE.test(n) && !isCrudePlayerName(n) && !nameKey(n).startsWith("bot-") && !taken(n);
  const base = (wanted ?? "").replace(/[^\p{L}\p{N}_\- ]+/gu, "").replace(/\s+/g, " ").trim().slice(0, 19).trim();
  if (base && NAME_RE.test(base) && !isCrudePlayerName(base) && !nameKey(base).startsWith("bot-")) {
    if (ok(base)) return base;
    for (let i = 0; i < 50; i++) {
      const n = `${base} ${2 + rnd(9998)}`;
      if (ok(n)) return n;
    }
  }
  for (let i = 0; ; i++) {
    const n = `${pick(FIRST, rnd)} ${pick(SECOND, rnd)} ${1 + rnd(i < 50 ? 999 : 99999)}`;
    if (ok(n)) return n;
  }
}

function newSession(store: MvpStore, player: PlayerRef, now: Date): PlayerSession {
  const token = randomBytes(24).toString("base64url");
  store.addSession(player.id, hashToken(token), now.toISOString());
  return { player, token };
}

/** A new guest player, marked as one, and a session for this device. */
export function joinGuest(store: MvpStore, now: Date, rnd?: (n: number) => number): PlayerSession {
  const player: PlayerRef = { id: randomUUID(), name: freeName(store, undefined, rnd), bot: false };
  store.addPlayer(player);
  store.addGuest(player.id, now.toISOString());
  return newSession(store, player, now);
}

export interface CgLogin extends PlayerSession {
  /** "found": the account's player; "linked": this device's guest got the account; "new": a new player. */
  via: "found" | "linked" | "new";
}

/** The player of this CrazyGames account, and a session for this device:
 * the linked one; else `guest` (this device's player, by its session token)
 * when a guest join made it and it has no link yet, renamed to the CrazyGames username; else a new player, after `admit`
 * (the signup limit) says yes (false: undefined). */
export function loginCrazyGames(store: MvpStore, claims: CgClaims, guest: PlayerRef | undefined, now: Date, admit: () => boolean): CgLogin | undefined {
  const link = store.crazyGamesLink({ cgUserId: claims.userId });
  const linked = link && store.player(link.playerId);
  if (linked) return { ...newSession(store, linked, now), via: "found" };
  const at = now.toISOString();
  if (guest && !guest.bot && store.isGuest(guest.id) && !store.crazyGamesLink({ playerId: guest.id })) {
    const named = nameKey(guest.name) === nameKey(claims.username.trim()) ? guest : { ...guest, name: freeName(store, claims.username) };
    store.linkCrazyGames({ cgUserId: claims.userId, playerId: guest.id, linkedAt: at });
    if (named !== guest) store.addPlayer(named);
    return { ...newSession(store, named, now), via: "linked" };
  }
  if (!admit()) return undefined;
  const player: PlayerRef = { id: randomUUID(), name: freeName(store, claims.username), bot: false };
  store.addPlayer(player);
  store.linkCrazyGames({ cgUserId: claims.userId, playerId: player.id, linkedAt: at });
  return { ...newSession(store, player, now), via: "new" };
}

/** Players from CrazyGames (a guest join or a linked account) play without
 * the email an invite-only server asks of the others. */
export const isCrazyGamesPlayer = (store: MvpStore, playerId: string) => store.isGuest(playerId) || !!store.crazyGamesLink({ playerId });
