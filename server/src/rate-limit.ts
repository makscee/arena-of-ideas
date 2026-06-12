/**
 * In-memory sliding-window rate limiter. Copied from void-auth
 * (src/lib/rate-limit.ts).
 *
 * Per-key, stores the timestamps of the last N hits. On each call, drops
 * timestamps older than `now - windowMs`, counts remaining; if count >= limit,
 * rejects with retryAfterMs = oldest + windowMs - now. Otherwise pushes now
 * and accepts.
 *
 * Process-local memory; lost on restart, not shared across replicas. Fine for
 * a single-container arena server.
 */

export type RateLimitResult = { ok: true } | { ok: false; retryAfterMs: number };

export interface RateLimiterOptions {
  limit: number;
  windowMs: number;
  /** Injectable clock for tests; defaults to Date.now. Milliseconds. */
  clock?: () => number;
}

export interface RateLimiter {
  check(key: string): RateLimitResult;
}

export function createRateLimiter(opts: RateLimiterOptions): RateLimiter {
  const { limit, windowMs } = opts;
  const clock = opts.clock ?? (() => Date.now());
  const buckets = new Map<string, number[]>();

  function evict(arr: number[], cutoff: number): number[] {
    // arr is monotonically increasing; drop leading entries < cutoff.
    let i = 0;
    while (i < arr.length && arr[i]! < cutoff) i++;
    return i === 0 ? arr : arr.slice(i);
  }

  function sweepStale(cutoff: number): void {
    // Lazy GC: evict stale entries across all buckets and drop any that go
    // empty. Bounded by total tracked keys; amortized over check() calls.
    for (const [k, v] of buckets) {
      if (v.length === 0 || v[v.length - 1]! < cutoff) {
        buckets.delete(k);
        continue;
      }
      if (v[0]! < cutoff) {
        const live = evict(v, cutoff);
        if (live.length === 0) buckets.delete(k);
        else buckets.set(k, live);
      }
    }
  }

  return {
    check(key: string): RateLimitResult {
      const now = clock();
      const cutoff = now - windowMs;

      sweepStale(cutoff);

      const existing = buckets.get(key) ?? [];

      if (existing.length >= limit) {
        const oldest = existing[0]!;
        return { ok: false, retryAfterMs: oldest + windowMs - now };
      }

      existing.push(now);
      buckets.set(key, existing);

      return { ok: true };
    },
  };
}
