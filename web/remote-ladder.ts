// Remote ladder (PRD #016 slice 3) — the shared server ladder behind the
// kernel's LadderStore interface, so the run screen and the leaderboard keep
// reading the seam they already bind to (#015 kept them backing-agnostic).
//
// Two kinds of read live here, and they must never blur:
//
// FIGHT reads are the contract reads (server/README.md): before each ladder
// fight the run screen awaits serve(runId, round) — GET /v1/runs/:id/pool/:r —
// and this store pins poolAt(round)/champion() to THAT ONE response until the
// next serve. The server records every view it hands out and submission replay
// accepts only recorded views, so the fight must draw from exactly what was
// served: pool and champion from one response, never mixed across reads.
// ladderFight's own writes during the fight (addSnapshot of the run's ghost,
// setChampion on a crown) land in this client-side view only — the server
// re-derives and writes the real ghosts/crowns at submit.
//
// DISPLAY reads are everything else: the leaderboard and the run screen's side
// ladder read public pools + champion synced via sync(). The run's own
// not-yet-submitted ghosts ride along in a local overlay so a player sees
// their climb; clearLocal() drops the overlay once a submit lands (the server
// copy takes over) or a new run opens.

import type { LadderStore, TeamSnapshot } from "../src/index.js";
// The seq precondition and clone-on-write come from the kernel's ladder
// module itself (not re-exported via index) — same semantics as any backing.
import { assertSeqInOrder, jsonClone } from "../src/ladder.js";
import type { ArenaApi, SubmitInfo } from "./api.js";

/** Matches the leaderboard's scan cap — pools are contiguous from round 1. */
const SYNC_ROUND_CAP = 200;

export type RemoteResult = { ok: true } | { ok: false; reason: string };

export type RemoteSubmitResult =
  | { ok: true; crowned: boolean }
  | { ok: false; kind: "rejected" | "network"; reason: string };

/** The run screen's remote seam: mint/open/serve/submit, per the README's
 * open → serve → submit protocol. All methods resolve, never reject. */
export interface RemoteRun {
  /** A globally-unique runId — the README warns `run-<n>` collides. */
  mintRunId(): string;
  open(runId: string): Promise<RemoteResult>;
  /** The play read: pins the fight view this store serves poolAt/champion
   * from. Must resolve ok before every ladderFight; re-reads are free. */
  serve(runId: string, round: number): Promise<RemoteResult>;
  submit(run: string): Promise<RemoteSubmitResult>;
}

export class RemoteLadder implements LadderStore, RemoteRun {
  private readonly api: ArenaApi;
  private readonly token: string;
  // Display state, refreshed by sync(): the public pools and champion.
  private pools = new Map<number, TeamSnapshot[]>();
  private champ: TeamSnapshot | null = null;
  private champHolder: string | null = null;
  // The active run's local writes: its own ghosts (server gets them only at
  // submit) and a locally-won crown — display until the server copy lands.
  private localGhosts = new Map<number, TeamSnapshot[]>();
  private localCrown: TeamSnapshot | null = null;
  // The pinned fight view — the last serve() response, the only thing a
  // ladderFight may read. Kept after the fight so the shop's "rivals waiting"
  // line for that round reads the served (own-ghost-excluded) truth.
  private fight: { round: number; pool: TeamSnapshot[]; champion: TeamSnapshot } | null = null;

  constructor(api: ArenaApi, token: string) {
    this.api = api;
    this.token = token;
  }

  // ---------- LadderStore (what the screens and ladderFight read) ----------

  poolAt(round: number): readonly TeamSnapshot[] {
    if (this.fight !== null && this.fight.round === round) return this.fight.pool;
    const pub = this.pools.get(round) ?? [];
    const own = this.localGhosts.get(round) ?? [];
    return own.length === 0 ? pub : [...pub, ...own];
  }

  addSnapshot(snap: TeamSnapshot): void {
    if (this.fight !== null && this.fight.round === snap.round) {
      // Inside a fight: the seq precondition holds against the SERVED view —
      // that is the prefix length the server recorded and replay will demand.
      assertSeqInOrder(snap, this.fight.pool.length);
      this.fight.pool.push(jsonClone(snap));
    }
    // Either way the ghost joins the display overlay, so the side ladder and
    // leaderboard show the climb before the server has it.
    const own = this.localGhosts.get(snap.round) ?? [];
    own.push(jsonClone(snap));
    this.localGhosts.set(snap.round, own);
  }

  champion(): TeamSnapshot | null {
    // Fight view first: a champion challenge must name the champion co-served
    // with the claimed pool view, never a fresher display read.
    if (this.fight !== null) return this.fight.champion;
    return this.localCrown ?? this.champ;
  }

  setChampion(snap: TeamSnapshot): void {
    // Local display only — whether the crown really lands is the server's
    // call at submit (the crown race; `crowned: false` when it lapsed).
    this.localCrown = jsonClone(snap);
  }

  /** The champion holder's display name (public read) — null for bootstrap
   * or while a local, not-yet-confirmed crown is showing. While a fight view
   * is pinned, champion() reads the co-served champion: the name still
   * applies when it is the same champion the last sync described, and is
   * unknown (null) when the seat changed in between. */
  holder(): string | null {
    if (this.localCrown !== null) return null;
    if (this.fight !== null) {
      return this.champ !== null && this.fight.champion.runId === this.champ.runId ? this.champHolder : null;
    }
    return this.champHolder;
  }

  // ---------- display sync ----------

  /** Refresh the public pools + champion. Pools are contiguous from round 1,
   * so the walk stops at the first empty one. Partial failure leaves the
   * previous display state standing — stale beats blank. */
  async sync(): Promise<RemoteResult> {
    const champ = await this.api.champion();
    if (!champ.ok) return { ok: false, reason: failureReason(champ) };
    const pools = new Map<number, TeamSnapshot[]>();
    for (let round = 1; round <= SYNC_ROUND_CAP; round++) {
      const res = await this.api.pool(round);
      if (!res.ok) return { ok: false, reason: failureReason(res) };
      if (res.value.pool.length === 0) break;
      pools.set(round, res.value.pool);
    }
    this.champ = champ.value.champion;
    this.champHolder = champ.value.holder;
    this.pools = pools;
    return { ok: true };
  }

  /** Drop the active run's local view — on a new open (stale overlay belongs
   * to the previous run) and after an accepted submit (the next sync carries
   * the server's re-derived copy). */
  clearLocal(): void {
    this.fight = null;
    this.localGhosts = new Map();
    this.localCrown = null;
  }

  // ---------- RemoteRun (the open → serve → submit protocol) ----------

  mintRunId(): string {
    return `web-${crypto.randomUUID()}`;
  }

  async open(runId: string): Promise<RemoteResult> {
    const res = await this.api.openRun(this.token, runId);
    if (!res.ok) return { ok: false, reason: failureReason(res) };
    this.clearLocal();
    return { ok: true };
  }

  async serve(runId: string, round: number): Promise<RemoteResult> {
    const res = await this.api.servePool(this.token, runId, round);
    if (!res.ok) return { ok: false, reason: failureReason(res) };
    this.fight = {
      round: res.value.round,
      pool: [...res.value.pool],
      champion: res.value.champion,
    };
    return { ok: true };
  }

  async submit(run: string): Promise<RemoteSubmitResult> {
    const res = await this.api.submitRun(this.token, run);
    if (res.ok) {
      this.clearLocal();
      return { ok: true, crowned: (res.value as SubmitInfo).crowned };
    }
    if (res.kind === "rejected") return { ok: false, kind: "rejected", reason: res.reason };
    return { ok: false, kind: "network", reason: failureReason(res) };
  }
}

function failureReason(res: { ok: false; kind: string; reason?: string }): string {
  if (res.kind === "unauthorized") return "your session has expired — log in again";
  if (res.kind === "network") return `the server is unreachable (${res.reason ?? "no answer"})`;
  return res.reason ?? "the server refused";
}
