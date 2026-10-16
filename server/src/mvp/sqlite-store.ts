// The MVP's SQLite store (mission #574, slice 4): MvpStore on better-sqlite3,
// passed to mvpRuntime by main.ts. It passes the same contract as the memory
// store (./store.contract.ts).
//
// Migrations: every file in ./sql/, in filename order (<slice>-<name>.sql,
// e.g. 04-runs.sql), runs once and is recorded by name in mvp_migrations. A
// slice that needs a new table or column adds its own new file; it never
// edits one that has shipped, since a recorded file doesn't run again.
import Database from "better-sqlite3";
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { BattleRecord, Champion, DayState, FightKind, FusionDiscovery, Ghost, Idea, IdeaState, PlayerRef, PlayoffResult, Rating, Slay, UnitId } from "../../../src/mvp/contract.js";
import type { CodexClaim, CodexPair } from "../../../src/mvp/codex.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { MAX_SESSIONS, NO_IDEAS, nameKey, type DayTallies, type IdeaCounts, type Invite, type MvpStore, type PoolSnapshot, type PoolStint, type StoredUnit, type TelegramCode, type CrazyGamesLink, type TelegramLink, type UnitDayTally, type UnitStatus, type UnitTallies, type UnitTally, type Vote, type WordRequest, type CodexWish, type FoundWord, type DailyPost, type EmailCode, type EmailLink } from "./store.js";

const SQL_DIR = fileURLToPath(new URL("./sql/", import.meta.url));

/** Applies the ./sql/ files not yet recorded, each in its own transaction.
 * Returns the names it applied. */
export function migrateMvp(db: Database.Database, dir = SQL_DIR): string[] {
  db.exec("CREATE TABLE IF NOT EXISTS mvp_migrations (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL)");
  const done = new Set((db.prepare("SELECT name FROM mvp_migrations").all() as { name: string }[]).map((r) => r.name));
  const applied: string[] = [];
  for (const name of readdirSync(dir).filter((f) => f.endsWith(".sql")).sort()) {
    if (done.has(name)) continue;
    const sql = readFileSync(`${dir}/${name}`, "utf8");
    db.transaction(() => {
      db.exec(sql);
      db.prepare("INSERT INTO mvp_migrations (name, applied_at) VALUES (?, ?)").run(name, new Date().toISOString());
    })();
    applied.push(name);
  }
  return applied;
}

type Row = { json: string };
const parse = <T>(row: unknown): T | undefined => (row ? (JSON.parse((row as Row).json) as T) : undefined);
/** Runs stored before the bench (round 3) have no `bench`: an empty one
 * (their rules have no benchSize, so it stays empty). */
const withBench = (r: MvpRunState | undefined): MvpRunState | undefined => {
  if (r) r.bench ??= [];
  return r;
};
const parseAll = <T>(rows: unknown[]): T[] => rows.map((r) => JSON.parse((r as Row).json) as T);

export class SqliteMvpStore implements MvpStore {
  readonly db: Database.Database;

  /** `path` is a file, or ":memory:" for tests. `sqlDir`: the migrations
   * (tests build an older schema from a copy of fewer files). */
  constructor(path: string, sqlDir = SQL_DIR) {
    this.db = new Database(path);
    this.db.pragma("journal_mode = WAL");
    this.db.pragma("busy_timeout = 5000");
    migrateMvp(this.db, sqlDir);
  }

  close(): void { this.db.close(); }

  private one<T>(sql: string, ...args: unknown[]): T | undefined { return parse<T>(this.db.prepare(sql).get(...args)); }
  private all<T>(sql: string, ...args: unknown[]): T[] { return parseAll<T>(this.db.prepare(sql).all(...args)); }
  private write(sql: string, ...args: unknown[]): void { this.db.prepare(sql).run(...args); }

  addPlayer(p: PlayerRef): void { this.write("INSERT OR REPLACE INTO mvp_players (id, json) VALUES (?, ?)", p.id, JSON.stringify(p)); }
  player(id: string): PlayerRef | undefined { return this.one("SELECT json FROM mvp_players WHERE id = ?", id); }

  putRun(r: MvpRunState): void {
    this.write(
      "INSERT INTO mvp_runs (id, player_id, over, json) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET player_id = excluded.player_id, over = excluded.over, json = excluded.json",
      r.runId, r.player.id, r.phase === "over" ? 1 : 0, JSON.stringify(r),
    );
  }
  run(id: string): MvpRunState | undefined { return withBench(this.one("SELECT json FROM mvp_runs WHERE id = ?", id)); }
  activeRun(playerId: string): MvpRunState | undefined { return withBench(this.one("SELECT json FROM mvp_runs WHERE player_id = ? AND over = 0 ORDER BY rowid LIMIT 1", playerId)); }

  addGhost(g: Ghost): void {
    this.write("INSERT INTO mvp_ghosts (ghost_id, round, player_id, content_version, json) VALUES (?, ?, ?, ?, ?)", g.ghostId, g.round, g.player.id, g.contentVersion, JSON.stringify(g));
  }
  ghosts(round: number, opts: { excludePlayerId: string; limit?: number }): Ghost[] {
    // The newest `limit` (-1: all) through the (round, seq) index (M2-2), then
    // oldest first.
    return this.all(
      "SELECT json FROM (SELECT seq, json FROM mvp_ghosts WHERE round = ? AND player_id != ? ORDER BY seq DESC LIMIT ?) ORDER BY seq",
      round, opts.excludePlayerId, opts.limit ?? -1,
    );
  }

  putBattle(b: BattleRecord): void { this.write("INSERT OR REPLACE INTO mvp_battles (id, kind, at, json) VALUES (?, ?, ?, ?)", b.battleId, b.kind, b.at, JSON.stringify(b)); }
  battle(id: string): BattleRecord | undefined { return this.one("SELECT json FROM mvp_battles WHERE id = ?", id); }
  battles(opts: { kind?: FightKind; since?: string } = {}): BattleRecord[] {
    return this.all(
      "SELECT json FROM mvp_battles WHERE (@kind IS NULL OR kind = @kind) AND (@since IS NULL OR at >= @since) ORDER BY at, rowid",
      { kind: opts.kind ?? null, since: opts.since ?? null },
    );
  }

  fusion(first: UnitId, second: UnitId): FusionDiscovery | undefined { return this.one("SELECT json FROM mvp_fusions WHERE first = ? AND second = ?", first, second); }
  putFusion(f: FusionDiscovery): void {
    this.write("INSERT INTO mvp_fusions (first, second, json) VALUES (?, ?, ?) ON CONFLICT(first, second) DO UPDATE SET json = excluded.json", f.first, f.second, JSON.stringify(f));
  }
  fusions(): FusionDiscovery[] { return this.all("SELECT json FROM mvp_fusions ORDER BY rowid"); }

  codexPair(a: UnitId, b: UnitId): CodexPair | undefined { return this.one("SELECT json FROM mvp_codex_pairs WHERE a = ? AND b = ?", a, b); }
  putCodexPair(p: CodexPair): void {
    this.write("INSERT INTO mvp_codex_pairs (a, b, claimed, json) VALUES (?, ?, ?, ?) ON CONFLICT(a, b) DO UPDATE SET claimed = excluded.claimed, json = excluded.json", p.a, p.b, p.claim ? 1 : 0, JSON.stringify(p));
  }
  claimCodexPair(a: UnitId, b: UnitId, claim: CodexClaim): boolean {
    // One statement: only an unclaimed pair with candidates takes the claim.
    const r = this.db
      .prepare("UPDATE mvp_codex_pairs SET claimed = 1, json = json_set(json, '$.claim', json(?)) WHERE a = ? AND b = ? AND claimed = 0 AND json_array_length(json, '$.candidates') > 0")
      .run(JSON.stringify(claim), a, b);
    return r.changes === 1;
  }
  codexPairs(): CodexPair[] { return this.all("SELECT json FROM mvp_codex_pairs ORDER BY a, b"); }
  addWish(w: CodexWish): boolean {
    return this.db.prepare("INSERT OR IGNORE INTO mvp_codex_wishes (a, b, player_id, seq, at) VALUES (?, ?, ?, ?, ?)").run(w.a, w.b, w.playerId, w.seq, w.at).changes === 1;
  }
  wishes(seq: number): { a: UnitId; b: UnitId; count: number }[] {
    return this.db.prepare("SELECT a, b, COUNT(*) AS count FROM mvp_codex_wishes WHERE seq = ? GROUP BY a, b ORDER BY count DESC, a, b").all(seq) as { a: UnitId; b: UnitId; count: number }[];
  }
  wishedBy(a: UnitId, b: UnitId, playerId: string, seq: number): boolean {
    return this.db.prepare("SELECT 1 FROM mvp_codex_wishes WHERE a = ? AND b = ? AND player_id = ? AND seq = ?").get(a, b, playerId, seq) !== undefined;
  }

  currentDay(): DayState | undefined { return this.one("SELECT json FROM mvp_days ORDER BY seq DESC LIMIT 1"); }
  putDay(d: DayState): void { this.write("INSERT OR REPLACE INTO mvp_days (seq, json) VALUES (?, ?)", d.seq, JSON.stringify(d)); }

  champion(seq: number): Champion | undefined { return this.one("SELECT json FROM mvp_champions WHERE seq = ?", seq); }
  currentChampion(): Champion | undefined { return this.one("SELECT json FROM mvp_champions ORDER BY seq DESC LIMIT 1"); }
  putChampion(c: Champion): void { this.write("INSERT OR REPLACE INTO mvp_champions (seq, json) VALUES (?, ?)", c.seq, JSON.stringify(c)); }
  champions(): Champion[] { return this.all("SELECT json FROM mvp_champions ORDER BY seq"); }

  addSlay(s: Slay): void { this.write("INSERT INTO mvp_slays (seq, json) VALUES (?, ?)", s.seq, JSON.stringify(s)); }
  slays(seq: number): Slay[] { return this.all("SELECT json FROM mvp_slays WHERE seq = ? ORDER BY id", seq); }

  playoff(seq: number): PlayoffResult | undefined { return this.one("SELECT json FROM mvp_playoffs WHERE seq = ?", seq); }
  putPlayoff(p: PlayoffResult): void { this.write("INSERT OR REPLACE INTO mvp_playoffs (seq, json) VALUES (?, ?)", p.seq, JSON.stringify(p)); }

  rating(playerId: string): Rating | undefined { return this.one("SELECT json FROM mvp_ratings WHERE player_id = ?", playerId); }
  putRating(r: Rating): void { this.write("INSERT OR REPLACE INTO mvp_ratings (player_id, json) VALUES (?, ?)", r.player.id, JSON.stringify(r)); }

  addUnitTallies(contentVersion: string, delta: UnitTallies): void {
    this.db.transaction(() => {
      if (delta.runs) {
        this.write(
          "INSERT INTO mvp_run_tallies (content_version, runs) VALUES (?, ?) ON CONFLICT(content_version) DO UPDATE SET runs = runs + excluded.runs",
          contentVersion, delta.runs,
        );
      }
      const up = this.db.prepare(
        "INSERT INTO mvp_unit_tallies (content_version, unit_id, fights, wins, runs) VALUES (?, ?, ?, ?, ?) " +
          "ON CONFLICT(content_version, unit_id) DO UPDATE SET fights = fights + excluded.fights, wins = wins + excluded.wins, runs = runs + excluded.runs",
      );
      for (const u of delta.units) up.run(contentVersion, u.unitId, u.fights, u.wins, u.runs);
    })();
  }
  unitTallies(contentVersion: string): UnitTallies {
    const runs = (this.db.prepare("SELECT runs FROM mvp_run_tallies WHERE content_version = ?").get(contentVersion) as { runs: number } | undefined)?.runs ?? 0;
    const units = this.db
      .prepare("SELECT unit_id AS unitId, fights, wins, runs FROM mvp_unit_tallies WHERE content_version = ? ORDER BY unit_id")
      .all(contentVersion) as UnitTally[];
    return { runs, units };
  }

  playersNamed(name: string, opts: { bots?: boolean } = {}): PlayerRef[] {
    // Compared in JS (nameKey): SQLite's lower() folds ASCII only.
    const key = nameKey(name);
    return this.all<PlayerRef>(`SELECT json FROM mvp_players${opts.bots ? "" : " WHERE json_extract(json, '$.bot') = 0"} ORDER BY rowid`).filter((p) => nameKey(p.name) === key);
  }
  putInvite(i: Invite): void {
    this.write(
      "INSERT INTO mvp_invites (code, name_key, player_id, json) VALUES (?, ?, ?, ?) ON CONFLICT(code) DO UPDATE SET name_key = excluded.name_key, player_id = excluded.player_id, json = excluded.json",
      i.code, nameKey(i.name), i.playerId, JSON.stringify(i),
    );
  }
  invite(code: string): Invite | undefined { return this.one("SELECT json FROM mvp_invites WHERE code = ?", code); }
  invites(): Invite[] { return this.all("SELECT json FROM mvp_invites ORDER BY rowid"); }
  redeemInvite(code: string, tokenHash: string, at: string): string | undefined {
    // IMMEDIATE takes the write lock first, and the session row is written
    // only from the invite row as it is then: a revoke on another connection
    // either ran before (no row, no session) or runs after (and ends it).
    return this.db.transaction(() => {
      const added = this.db
        .prepare("INSERT INTO mvp_sessions (token_hash, player_id, created_at) SELECT ?, player_id, ? FROM mvp_invites WHERE code = ?")
        .run(tokenHash, at, code).changes;
      if (!added) return undefined;
      const { player_id: playerId } = this.db.prepare("SELECT player_id FROM mvp_invites WHERE code = ?").get(code) as { player_id: string };
      this.write("UPDATE mvp_invites SET json = json_set(json, '$.redeemedAt', ?) WHERE code = ? AND json_extract(json, '$.redeemedAt') IS NULL", at, code);
      this.write(
        "DELETE FROM mvp_sessions WHERE player_id = ? AND rowid NOT IN (SELECT rowid FROM mvp_sessions WHERE player_id = ? ORDER BY rowid DESC LIMIT ?)",
        playerId, playerId, MAX_SESSIONS,
      );
      return playerId;
    }).immediate();
  }
  sessionPlayer(tokenHash: string): string | undefined {
    return (this.db.prepare("SELECT player_id FROM mvp_sessions WHERE token_hash = ?").get(tokenHash) as { player_id: string } | undefined)?.player_id;
  }
  joinCode(): string | undefined {
    return (this.db.prepare("SELECT value FROM mvp_settings WHERE key = 'join_code'").get() as { value: string } | undefined)?.value;
  }
  setJoinCode(code: string): void { this.write("INSERT INTO mvp_settings (key, value) VALUES ('join_code', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value", code); }
  addSession(playerId: string, tokenHash: string, at: string): void {
    this.db.transaction(() => {
      this.write("INSERT INTO mvp_sessions (token_hash, player_id, created_at) VALUES (?, ?, ?)", tokenHash, playerId, at);
      this.write(
        "DELETE FROM mvp_sessions WHERE player_id = ? AND rowid NOT IN (SELECT rowid FROM mvp_sessions WHERE player_id = ? ORDER BY rowid DESC LIMIT ?)",
        playerId, playerId, MAX_SESSIONS,
      );
    }).immediate();
  }
  putTelegramCode(c: TelegramCode): void {
    this.write("INSERT INTO mvp_telegram_codes (code_hash, json) VALUES (?, ?) ON CONFLICT(code_hash) DO UPDATE SET json = excluded.json", c.codeHash, JSON.stringify(c));
  }
  telegramCode(codeHash: string): TelegramCode | undefined { return this.one("SELECT json FROM mvp_telegram_codes WHERE code_hash = ?", codeHash); }
  dropTelegramCode(codeHash: string): boolean { return this.db.prepare("DELETE FROM mvp_telegram_codes WHERE code_hash = ?").run(codeHash).changes > 0; }
  linkTelegram(l: TelegramLink): void {
    // Both keys are unique: a second link of either side fails here.
    this.write("INSERT INTO mvp_telegram_links (tg_user_id, player_id, json) VALUES (?, ?, ?)", l.tgUserId, l.playerId, JSON.stringify(l));
  }
  telegramLink(by: { tgUserId: string } | { playerId: string }): TelegramLink | undefined {
    return "tgUserId" in by
      ? this.one("SELECT json FROM mvp_telegram_links WHERE tg_user_id = ?", by.tgUserId)
      : this.one("SELECT json FROM mvp_telegram_links WHERE player_id = ?", by.playerId);
  }
  linkCrazyGames(l: CrazyGamesLink): void {
    this.write("INSERT INTO mvp_cg_links (cg_user_id, player_id, json) VALUES (?, ?, ?)", l.cgUserId, l.playerId, JSON.stringify(l));
  }
  crazyGamesLink(by: { cgUserId: string } | { playerId: string }): CrazyGamesLink | undefined {
    return "cgUserId" in by
      ? this.one("SELECT json FROM mvp_cg_links WHERE cg_user_id = ?", by.cgUserId)
      : this.one("SELECT json FROM mvp_cg_links WHERE player_id = ?", by.playerId);
  }
  addGuest(playerId: string, at: string): void {
    this.write("INSERT OR IGNORE INTO mvp_guests (player_id, created_at) VALUES (?, ?)", playerId, at);
  }
  isGuest(playerId: string): boolean {
    return !!this.db.prepare("SELECT 1 FROM mvp_guests WHERE player_id = ?").get(playerId);
  }
  unlinkTelegram(playerId: string): boolean { return this.db.prepare("DELETE FROM mvp_telegram_links WHERE player_id = ?").run(playerId).changes > 0; }
  linkEmail(l: EmailLink): void {
    // Both keys are unique: a second email of either side fails here.
    this.write("INSERT INTO mvp_emails (email_key, player_id, json) VALUES (?, ?, ?)", l.emailKey, l.playerId, JSON.stringify(l));
  }
  emailLink(by: { emailKey: string } | { playerId: string }): EmailLink | undefined {
    return "emailKey" in by
      ? this.one("SELECT json FROM mvp_emails WHERE email_key = ?", by.emailKey)
      : this.one("SELECT json FROM mvp_emails WHERE player_id = ?", by.playerId);
  }
  putEmailCode(c: EmailCode): void {
    this.write("INSERT INTO mvp_email_codes (email_key, json) VALUES (?, ?) ON CONFLICT(email_key) DO UPDATE SET json = excluded.json", c.emailKey, JSON.stringify(c));
  }
  emailCode(emailKey: string): EmailCode | undefined { return this.one("SELECT json FROM mvp_email_codes WHERE email_key = ?", emailKey); }
  dropEmailCode(emailKey: string): boolean { return this.db.prepare("DELETE FROM mvp_email_codes WHERE email_key = ?").run(emailKey).changes > 0; }
  rotateInvite(oldCode: string, next: Invite): number {
    return this.db.transaction(() => {
      this.write("DELETE FROM mvp_invites WHERE code = ?", oldCode);
      this.putInvite(next);
      return this.db.prepare("DELETE FROM mvp_sessions WHERE player_id = ?").run(next.playerId).changes;
    }).immediate();
  }

  putUnit(u: StoredUnit): void {
    this.write(
      "INSERT INTO mvp_units (unit_id, status, author_id, json) VALUES (?, ?, ?, ?) ON CONFLICT(unit_id) DO UPDATE SET status = excluded.status, author_id = excluded.author_id, json = excluded.json",
      u.unitId, u.status, u.authorId, JSON.stringify(u),
    );
  }
  unit(id: UnitId): StoredUnit | undefined { return this.one("SELECT json FROM mvp_units WHERE unit_id = ?", id); }
  units(opts: { status?: UnitStatus } = {}): StoredUnit[] {
    return this.all("SELECT json FROM mvp_units WHERE (@status IS NULL OR status = @status) ORDER BY rowid", { status: opts.status ?? null });
  }
  putPool(p: PoolSnapshot): void { this.write("INSERT INTO mvp_pools (version, day_seq, json) VALUES (?, ?, ?)", p.version, p.daySeq, JSON.stringify(p)); }
  pool(version: string): PoolSnapshot | undefined { return this.one("SELECT json FROM mvp_pools WHERE version = ? ORDER BY seq DESC LIMIT 1", version); }
  currentPool(): PoolSnapshot | undefined { return this.one("SELECT json FROM mvp_pools ORDER BY seq DESC LIMIT 1"); }
  putStint(s: PoolStint): void {
    this.write(
      "INSERT INTO mvp_pool_stints (unit_id, entered_seq, json) VALUES (?, ?, ?) ON CONFLICT(unit_id, entered_seq) DO UPDATE SET json = excluded.json",
      s.unitId, s.enteredSeq, JSON.stringify(s),
    );
  }
  stints(unitId?: UnitId): PoolStint[] {
    return this.all("SELECT json FROM mvp_pool_stints WHERE (@unit IS NULL OR unit_id = @unit) ORDER BY unit_id, entered_seq", { unit: unitId ?? null });
  }
  seedPool(units: StoredUnit[], pool: PoolSnapshot, stints: PoolStint[]): boolean {
    // IMMEDIATE: two servers starting on one file can't both seed.
    return this.db.transaction(() => {
      if (this.db.prepare("SELECT 1 FROM mvp_pools LIMIT 1").get()) return false;
      for (const u of units) this.putUnit(u);
      this.putPool(pool);
      for (const s of stints) this.putStint(s);
      return true;
    }).immediate();
  }
  atomically<T>(fn: () => T): T { return this.db.transaction(fn).immediate(); }
  addDayTallies(daySeq: number, delta: DayTallies): void {
    this.db.transaction(() => {
      if (delta.runs) {
        this.write("INSERT INTO mvp_day_run_tallies (day_seq, runs) VALUES (?, ?) ON CONFLICT(day_seq) DO UPDATE SET runs = runs + excluded.runs", daySeq, delta.runs);
      }
      const up = this.db.prepare(
        "INSERT INTO mvp_unit_day_tallies (day_seq, unit_id, fights, wins, runs, picks) VALUES (?, ?, ?, ?, ?, ?) " +
          "ON CONFLICT(day_seq, unit_id) DO UPDATE SET fights = fights + excluded.fights, wins = wins + excluded.wins, runs = runs + excluded.runs, picks = picks + excluded.picks",
      );
      for (const u of delta.units) up.run(daySeq, u.unitId, u.fights, u.wins, u.runs, u.picks);
    })();
  }
  dayTallies(daySeq: number): DayTallies {
    const runs = (this.db.prepare("SELECT runs FROM mvp_day_run_tallies WHERE day_seq = ?").get(daySeq) as { runs: number } | undefined)?.runs ?? 0;
    const units = this.db
      .prepare("SELECT unit_id AS unitId, fights, wins, runs, picks FROM mvp_unit_day_tallies WHERE day_seq = ? ORDER BY unit_id")
      .all(daySeq) as UnitDayTally[];
    return { runs, units };
  }
  ideaCounts(playerId: string): IdeaCounts { return this.one<IdeaCounts>("SELECT json FROM mvp_idea_counts WHERE player_id = ?", playerId) ?? { ...NO_IDEAS }; }
  putIdeaCounts(playerId: string, c: IdeaCounts): void {
    this.write("INSERT INTO mvp_idea_counts (player_id, json) VALUES (?, ?) ON CONFLICT(player_id) DO UPDATE SET json = excluded.json", playerId, JSON.stringify(c));
  }
  putIdea(i: Idea): void {
    this.write(
      "INSERT INTO mvp_ideas (id, player_id, text, state, created_at, json) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET player_id = excluded.player_id, text = excluded.text, state = excluded.state, created_at = excluded.created_at, json = excluded.json",
      i.ideaId, i.playerId, i.text, i.state, i.createdAt, JSON.stringify(i.data),
    );
  }
  idea(ideaId: string): Idea | undefined { return ideaOf(this.db.prepare("SELECT * FROM mvp_ideas WHERE id = ?").get(ideaId)); }
  ideas(opts: { playerId?: string; state?: IdeaState; target?: UnitId } = {}): Idea[] {
    return this.db
      .prepare("SELECT * FROM mvp_ideas WHERE (@player IS NULL OR player_id = @player) AND (@state IS NULL OR state = @state) AND (@target IS NULL OR json_extract(json, '$.target') = @target) ORDER BY rowid")
      .all({ player: opts.playerId ?? null, state: opts.state ?? null, target: opts.target ?? null })
      .map((r) => ideaOf(r)!);
  }
  deleteIdea(ideaId: string): void { this.write("DELETE FROM mvp_ideas WHERE id = ?", ideaId); }
  addVote(v: Vote): boolean {
    const r = this.db
      .prepare("INSERT OR IGNORE INTO mvp_votes (player_id, candidate_id, other_id, pick, created_at) VALUES (?, ?, ?, ?, ?)")
      .run(v.playerId, v.candidateId, v.otherId, v.pick, v.createdAt);
    return r.changes > 0;
  }
  votes(opts: { candidateId?: UnitId; playerId?: string } = {}): Vote[] {
    type VoteRow = { player_id: string; candidate_id: string; other_id: string; pick: string | null; created_at: string };
    return (this.db
      .prepare("SELECT * FROM mvp_votes WHERE (@candidate IS NULL OR candidate_id = @candidate) AND (@player IS NULL OR player_id = @player) ORDER BY rowid")
      .all({ candidate: opts.candidateId ?? null, player: opts.playerId ?? null }) as VoteRow[])
      .map((r) => ({ playerId: r.player_id, candidateId: r.candidate_id, otherId: r.other_id, pick: r.pick, createdAt: r.created_at }));
  }
  addWordRequest(w: WordRequest): void {
    // idea_id is the column from before the queue held other sources; it mirrors ref
    this.write("INSERT INTO mvp_word_requests (idea_id, source, ref, word, part, created_at) VALUES (?, ?, ?, ?, ?, ?)", w.ref, w.source, w.ref, w.word, w.part, w.createdAt);
  }
  addFoundWord(f: FoundWord): boolean {
    return this.db.prepare("INSERT OR IGNORE INTO mvp_words_found (key, word, kind, pair, player_id, found_at) VALUES (?, ?, ?, ?, ?, ?)").run(f.key, f.word, f.kind, f.pair, f.playerId, f.foundAt).changes > 0;
  }
  foundWords(): FoundWord[] {
    return this.db.prepare("SELECT key, word, kind, pair, player_id AS playerId, found_at AS foundAt FROM mvp_words_found ORDER BY found_at, key").all() as FoundWord[];
  }
  wordRequests(): WordRequest[] {
    return this.db.prepare("SELECT source, ref, word, part, created_at AS createdAt FROM mvp_word_requests ORDER BY id").all() as WordRequest[];
  }
  claimDailyPost(seq: number, key: string, at: string): boolean {
    return this.db.prepare("INSERT OR IGNORE INTO mvp_daily_posts (seq, key, state, tries, at) VALUES (?, ?, 'sending', 0, ?)").run(seq, key, at).changes > 0;
  }
  putDailyPost(p: DailyPost): void {
    // An update keeps the row's place (claim order); a row never claimed is added.
    const r = this.db.prepare("UPDATE mvp_daily_posts SET state = ?, tries = ?, at = ?, error = ? WHERE seq = ? AND key = ?").run(p.state, p.tries, p.at, p.error ?? null, p.seq, p.key);
    if (!r.changes) this.write("INSERT INTO mvp_daily_posts (seq, key, state, tries, at, error) VALUES (?, ?, ?, ?, ?, ?)", p.seq, p.key, p.state, p.tries, p.at, p.error ?? null);
  }
  dailyPosts(seq?: number): DailyPost[] {
    type PostRow = { seq: number; key: string; state: DailyPost["state"]; tries: number; at: string; error: string | null };
    return (this.db.prepare("SELECT * FROM mvp_daily_posts WHERE (@seq IS NULL OR seq = @seq) ORDER BY rowid").all({ seq: seq ?? null }) as PostRow[]).map((r) => ({
      seq: r.seq,
      key: r.key,
      state: r.state,
      tries: r.tries,
      at: r.at,
      ...(r.error === null ? {} : { error: r.error }),
    }));
  }
}

type IdeaRow = { id: string; player_id: string; text: string; state: IdeaState; created_at: string; json: string };
const ideaOf = (row: unknown): Idea | undefined => {
  const r = row as IdeaRow | undefined;
  return r && { ideaId: r.id, playerId: r.player_id, text: r.text, state: r.state, createdAt: r.created_at, data: JSON.parse(r.json) as Idea["data"] };
};
