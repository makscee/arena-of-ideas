// MVP storage seam (mission #574). Slice 1 keeps everything in memory behind
// this interface; slice 4 swaps in SQLite without touching the routes.
import type { BattleRecord, FusionDiscovery, Ghost, PlayerRef, UnitId } from "../../../src/mvp/contract.js";
import type { MvpRunState } from "../../../src/mvp/run.js";

export interface MvpStore {
  addPlayer(p: PlayerRef): void;
  player(id: string): PlayerRef | undefined;
  putRun(r: MvpRunState): void;
  run(id: string): MvpRunState | undefined;
  activeRun(playerId: string): MvpRunState | undefined;
  addGhost(g: Ghost): void;
  /** Saved teams at this round, from runs other than `excludeRunId`. */
  ghosts(round: number, excludeRunId: string): Ghost[];
  putBattle(b: BattleRecord): void;
  battle(id: string): BattleRecord | undefined;
  // Discovered fusions, one per ordered pair. Slice 10 owns them and is the
  // only writer (the credit rule is on FusionDiscovery); everyone else reads.
  fusion(first: UnitId, second: UnitId): FusionDiscovery | undefined;
  /** Stores or replaces the pair's discovery (replacing is how a human claims a bot's pair). */
  putFusion(f: FusionDiscovery): void;
  fusions(): FusionDiscovery[];
}

export class MemoryMvpStore implements MvpStore {
  private players = new Map<string, PlayerRef>();
  private runs = new Map<string, MvpRunState>();
  private ghostsByRound = new Map<number, Ghost[]>();
  private battles = new Map<string, BattleRecord>();
  private fusionsByPair = new Map<string, FusionDiscovery>();
  addPlayer(p: PlayerRef): void { this.players.set(p.id, p); }
  player(id: string): PlayerRef | undefined { return this.players.get(id); }
  putRun(r: MvpRunState): void { this.runs.set(r.runId, r); }
  run(id: string): MvpRunState | undefined { return this.runs.get(id); }
  activeRun(playerId: string): MvpRunState | undefined {
    for (const r of this.runs.values()) if (r.player.id === playerId && r.phase === "shop") return r;
    return undefined;
  }
  addGhost(g: Ghost): void {
    const list = this.ghostsByRound.get(g.round) ?? [];
    list.push(g);
    this.ghostsByRound.set(g.round, list);
  }
  ghosts(round: number, excludeRunId: string): Ghost[] {
    return (this.ghostsByRound.get(round) ?? []).filter((g) => g.runId !== excludeRunId);
  }
  putBattle(b: BattleRecord): void { this.battles.set(b.battleId, b); }
  battle(id: string): BattleRecord | undefined { return this.battles.get(id); }
  fusion(first: UnitId, second: UnitId): FusionDiscovery | undefined { return this.fusionsByPair.get(pairKey(first, second)); }
  putFusion(f: FusionDiscovery): void { this.fusionsByPair.set(pairKey(f.first, f.second), f); }
  fusions(): FusionDiscovery[] { return [...this.fusionsByPair.values()]; }
}

/** Ordered: (a, b) and (b, a) are different fusions. */
function pairKey(first: UnitId, second: UnitId): string {
  return JSON.stringify([first, second]);
}
