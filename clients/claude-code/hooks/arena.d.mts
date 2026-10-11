// What register.tsx uses of ./arena.mjs (the bundle of ../src/lib.ts), typed
// loosely: the real types are in src/pane.ts, clients/core and clients/mcp.
import type { Decision, Lang, Pending, RunView, Suggestion, View } from '../types'

type Fetch = (url: string, init?: { method?: string; headers?: Record<string, string>; body?: string }) => Promise<{ status: number; ok: boolean; text: string }>
export type Session = { token?: string; email?: string; name?: string }
export type SessionStore = { load(): Session; save(s: Session): void }
export type Tool = { name: string; description: string; inputSchema: any; run(args: Record<string, unknown>): Promise<string> }
export type Key = { key: string; label: string }
export type Press = { decision: Decision } | { pending: Pending | null } | { do: 'battle' | 'accept' | 'lang' | 'ranked' | 'practice' | 'back' | 'older' | 'newer' } | { none: true }

export const DEFAULT_URL: string
export function arenaTools(ctx: { url: string; token?: string | undefined; store: SessionStore; fetch?: any; client?: string }): Tool[]
export function failureText(err: unknown): string
export class ApiError extends Error {
  readonly status: number
}
export class ArenaClient {
  constructor(conn: { api: string; token?: string | null; lang?: string; client?: string; fetch?: any })
  conn: { token?: string | null }
  home(): Promise<any>
  content(): Promise<any>
  run(id: string): Promise<RunView & any>
  startRun(mode?: 'ranked' | 'practice'): Promise<RunView & any>
  decide(id: string, d: Decision): Promise<{ run: RunView & any; fight?: any }>
  battle(id: string): Promise<any>
  emailStart(email: string, name?: string): Promise<unknown>
  emailVerify(email: string, code: string, name?: string): Promise<{ status: 'done'; token: string; player: { id: string; name: string } } | { status: 'needsName' }>
}
export function apiBase(root: string): string
export function battleText(b: any, o?: { lang?: Lang; view?: 'summary' | 'steps'; you?: 'A' | 'B' }): string
export function keysFor(run: RunView | null, rules: any, pending: Pending | null, suggestion: Suggestion | null, view: View, o?: { lang?: Lang }): Key[]
export function heldKeys(keys: Key[]): string[]
export function press(key: string, run: RunView | null, rules: any, pending: Pending | null, suggestion: Suggestion | null, view: View): Press
export function decisionText(d: Decision, run: RunView, rules: any, content: any, o?: { lang?: Lang }): string
export function paneLines(run: RunView, content: any, rules: any, o?: { lang?: Lang }, wide?: boolean): string[]
export function battlePage(text: string, page: number, rows: number): { lines: string[]; page: number; pages: number }
export function hostFetch(f: Fetch): any
