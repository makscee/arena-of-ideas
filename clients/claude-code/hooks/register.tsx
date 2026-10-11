// Arena in Claude Code (makscee/void-board#960): a pane with the run, played
// with keys, and the session's Claude as a coach. Everything goes to the hosted
// API (ARENA_URL, default https://arena.makscee.ru); nothing runs locally.
//
// Assist only (#955): a key the player presses is a human move, sent without
// the agent header. Claude's tools are arena-mcp's (one code path, bundled in
// ./arena.mjs) and send X-Arena-Client: agent/claude-code, so the server
// itself refuses Claude a ranked move; Claude's `suggest` only shows its pick
// in the pane, and the player's `y` sends it.
import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register } from 'claude-code'

import type { Lang, Pending, RunView, Suggestion, View } from '../types'
import {
  ApiError,
  ArenaClient,
  DEFAULT_URL,
  apiBase,
  arenaTools,
  battlePage,
  battleText,
  decisionText,
  failureText,
  heldKeys,
  hostFetch,
  keysFor,
  paneLines,
  press,
  type Session,
  type Tool,
} from './arena.mjs'

const PANE = 'arena'
const AGENT = 'agent/claude-code'

const run = atom({ plugin: 'arena', key: 'run' } as const, null as RunView | null)
const status = atom({ plugin: 'arena', key: 'status' } as const, '')
const pending = atom({ plugin: 'arena', key: 'pending' } as const, null as Pending | null)
const suggestion = atom({ plugin: 'arena', key: 'suggestion' } as const, null as Suggestion | null)
const view = atom({ plugin: 'arena', key: 'view' } as const, 'run' as View)
const battle = atom({ plugin: 'arena', key: 'battle' } as const, null as { id: string; page: number } | null)
const lang = atom({ plugin: 'arena', key: 'lang' } as const, 'en' as Lang)
const busy = atom({ plugin: 'arena', key: 'busy' } as const, false)

type $ = EngineInterface

/** clients/core's fetch, over the host's network. */
const fetchVia = ($: $) => hostFetch((url, init) => $.http.fetch(url, init))

/** The server's root, and the token ARENA_TOKEN gives. */
async function where($: $) {
  const url = ((await $.env.get('ARENA_URL')) || DEFAULT_URL).replace(/\/$/, '').replace(/\/api\/v1$/, '')
  return { url, envToken: (await $.env.get('ARENA_TOKEN')) || undefined }
}

// The login, kept across sessions in $.store (one per server URL), and read
// once into `saved` because arena-mcp's store is synchronous.
let saved: { url: string; session: Session } | undefined
async function session($: $) {
  const { url, envToken } = await where($)
  if (saved?.url !== url) saved = { url, session: ((await $.store.get(`session:${url}`)) as Session | undefined) ?? {} }
  const save = (s: Session) => {
    saved = { url, session: s }
    void $.store.set(`session:${url}`, s)
  }
  return { url, token: envToken ?? saved.session.token, session: saved.session, save }
}

/** The player's own client: no agent header, so its moves are the player's. */
async function human($: $) {
  const s = await session($)
  return new ArenaClient({ api: apiBase(s.url), token: s.token ?? null, lang: await read($, lang), fetch: fetchVia($) })
}

// The pool and rules, fetched once per load (they change by the day; /arena fetches again).
let world: Promise<{ content: any; rules: any }> | undefined
const worldOf = ($: $) =>
  (world ??= (async () => {
    const c = await human($)
    const [content, home] = await Promise.all([c.content(), c.home()])
    return { content, rules: home.rules }
  })().catch(err => {
    world = undefined
    throw err
  }))

const say = (err: unknown) => (err instanceof ApiError && err.status === 401 ? 'Not logged in: /arena login <email>, then /arena code <code> (or set ARENA_TOKEN).' : failureText(err))

async function showRun($: $, r: RunView | null) {
  await update($, run, () => r)
  await update($, pending, () => null)
  await update($, suggestion, s => (s && r && s.runId === r.runId && r.phase !== 'over' ? s : null))
}

/** Reads the run shown again (Claude may have moved it). */
async function refresh($: $) {
  const r = await read($, run)
  if (!r) return
  try {
    await showRun($, await (await human($)).run(r.runId))
  } catch (err) {
    await update($, status, () => say(err))
  }
}

/** The run to show on open: the one shown, else the ranked run going. */
async function load($: $, mode?: 'ranked' | 'practice', start = false) {
  await update($, busy, () => true)
  try {
    const c = await human($)
    const home = await c.home()
    if (!home.rating) return void (await update($, status, () => 'Not logged in: /arena login <email>, then /arena code <code> (or set ARENA_TOKEN).'))
    const shown = await read($, run)
    const id = mode === 'practice' ? home.practiceRunId : mode === 'ranked' ? home.activeRunId : shown && shown.phase !== 'over' ? shown.runId : (home.activeRunId ?? home.practiceRunId)
    const r = id ? await c.run(id) : start && mode ? await c.startRun(mode) : null
    await showRun($, r)
    await update($, view, () => 'run' as View)
    await update($, status, () => (r ? '' : 'No run going: n starts a ranked run, p a practice run.'))
  } catch (err) {
    await update($, status, () => say(err))
  } finally {
    await update($, busy, () => false)
  }
}

/** A move the player made (a key, or `y` on Claude's pick). */
async function move($: $, d: any) {
  const r = await read($, run)
  if (!r || (await read($, busy))) return
  await update($, busy, () => true)
  try {
    const got = await (await human($)).decide(r.runId, d)
    await showRun($, got.run)
    await update($, suggestion, () => null) // Claude's pick was for the board before this move
    const ru = (await read($, lang)) === 'ru'
    await update($, status, () => (got.fight ? (ru ? 'b: повтор боя' : 'b: replay the fight') : ''))
  } catch (err) {
    await update($, pending, () => null)
    await update($, status, () => say(err))
  } finally {
    await update($, busy, () => false)
  }
}

async function onKey($: $, key: string) {
  const r = await read($, run)
  const w = await worldOf($).catch(() => null)
  const p = press(key, r, w?.rules ?? null, await read($, pending), await read($, suggestion), await read($, view))
  if ('decision' in p) return move($, p.decision)
  if ('pending' in p) return void (await update($, pending, () => p.pending))
  if (!('do' in p)) return
  await update($, status, () => '') // a hint from the last move ("b: replay the fight") is spent
  switch (p.do) {
    case 'lang':
      await update($, lang, l => (l === 'ru' ? 'en' : 'ru'))
      return void $.store.set('lang', await read($, lang))
    case 'ranked':
    case 'practice':
      return load($, p.do, true)
    case 'battle': {
      const last = r?.fights.at(-1)
      if (!last) return
      await update($, battle, () => ({ id: last.battleId, page: 0 }))
      return void (await update($, view, () => 'battle' as View))
    }
    case 'back':
      return void (await update($, view, () => 'run' as View))
    case 'older':
    case 'newer':
      return void (await update($, battle, b => (b ? { ...b, page: Math.max(0, b.page + (p.do === 'older' ? -1 : 1)) } : b)))
    case 'accept': {
      const s = await read($, suggestion)
      if (s) {
        await update($, suggestion, () => null)
        return move($, s.decision)
      }
    }
  }
}

// Battles as text, fetched once each (a battle never changes).
const battles = new Map<string, Promise<any>>()
const battleOf = async ($: $, id: string) => {
  let got = battles.get(id)
  if (!got) battles.set(id, (got = (async () => (await human($)).battle(id))().catch(err => (battles.delete(id), Promise.reject(err)))))
  return got
}

// Claude's tools: arena-mcp's, as the agent, plus `suggest`.
let tools: Tool[] | undefined
let toolsFor: string | undefined
async function agentTools($: $) {
  const s = await session($)
  const key = `${s.url} ${s.token ?? ''}`
  if (!tools || toolsFor !== key) {
    tools = arenaTools({ url: s.url, token: s.token, client: AGENT, fetch: fetchVia($), store: { load: () => saved?.session ?? {}, save: s.save } })
    toolsFor = key
  }
  return tools
}
const TOOL_NAMES = arenaTools({ url: DEFAULT_URL, store: { load: () => ({}), save: () => {} } }).map(t => ({ name: t.name, description: t.description, inputSchema: t.inputSchema }))
const RUN_TOOLS = new Set(['run', 'decide', 'preview'])
const SUGGEST = {
  name: 'suggest',
  description:
    "Shows your pick for the next move in the player's Arena pane, with a short why; the player sends it with y. It never moves anything itself. Use it on the ranked run the pane shows (decide only plays practice runs). Board numbers in a decision are 0-based, as the run tool prints them.",
  inputSchema: {
    type: 'object',
    properties: { decision: TOOL_NAMES.find(t => t.name === 'decide')!.inputSchema.properties.decision, why: { type: 'string', description: 'A few words: why this move.' } },
    required: ['decision', 'why'],
    additionalProperties: false,
  },
}

const open = ($: $) => $.ui.open({ id: PANE, title: 'Arena', focus: true })

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    const l = await $.store.get('lang')
    if (l === 'ru' || l === 'en') await update($, lang, () => l)
    await $.command.register({ name: 'arena', description: 'Play Arena of Ideas in a pane: /arena [practice|ranked|ru|en|login <email>|code <code> [name]|logout]' })
    for (const t of [...TOOL_NAMES, SUGGEST]) await $.tool.register({ name: t.name, description: t.description, inputSchema: t.inputSchema })
    return next(e)
  })

  on('command.run', { command: 'arena' }, async ($, e) => {
    const [word = '', ...rest] = e.args.trim().split(/\s+/).filter(Boolean)
    const s = await session($)
    try {
      switch (word) {
        case 'login': {
          if (!rest[0]) return { text: 'Usage: /arena login <email> [name]' }
          await (await human($)).emailStart(rest[0], rest[1])
          s.save({ ...s.session, email: rest[0], ...(rest[1] ? { name: rest[1] } : {}) })
          return { text: `Arena: a code was mailed to ${rest[0]}. Then: /arena code <code>${rest[1] ? '' : ' [name, for a new player]'}` }
        }
        case 'code': {
          if (!s.session.email) return { text: 'Arena: /arena login <email> first.' }
          const r = await (await human($)).emailVerify(s.session.email, rest[0] ?? '', rest[1] ?? s.session.name)
          if (r.status === 'needsName') return { text: 'Arena: a new player needs a name: /arena code <code> <name>' }
          s.save({ token: r.token })
          world = undefined
          await load($)
          await open($)
          return { text: `Arena: logged in as @${r.player.name}.` }
        }
        case 'logout':
          s.save({})
          await showRun($, null)
          return { text: 'Arena: logged out here.' }
        case 'ru':
        case 'en':
          await update($, lang, () => word as Lang)
          await $.store.set('lang', word)
          break
        case 'practice':
        case 'ranked':
          await load($, word, true)
          break
        default:
          world = undefined
          await load($)
      }
    } catch (err) {
      await update($, status, () => say(err))
    }
    await open($)
    const r = await read($, run)
    return { text: r ? `Arena: ${r.mode === 'practice' ? 'practice' : 'ranked'} run ${r.runId} in the pane.` : `Arena: ${(await read($, status)) || 'pane open.'}` }
  })

  // arena-mcp's tools (src/pane.spec.ts holds this list to theirs).
  on('tool.call', { tool: /^mcp__arena__(login|login_code|home|start_run|run|decide|preview|battle|trace|codex)$/ }, async ($, e) => {
    const t = TOOL_NAMES.find(x => `mcp__arena__${x.name}` === e.tool)!
    const args: Record<string, unknown> = {}
    for (const k of Object.keys((t.inputSchema as { properties?: object }).properties ?? {})) {
      const v = (e as unknown as Record<string, unknown>)[k]
      if (v !== undefined) args[k] = v
    }
    // The run in the pane is the one Claude means, unless it names another.
    const shown = await read($, run)
    if (RUN_TOOLS.has(t.name) && !args.runId && shown) args.runId = shown.runId
    if (!args.lang) args.lang = await read($, lang)
    const tool = (await agentTools($)).find(x => x.name === t.name)!
    try {
      const text = await tool.run(args)
      if (t.name === 'start_run') await load($, 'practice')
      else if (t.name === 'decide' || t.name === 'login_code') await refresh($)
      if (t.name === 'decide' && shown?.mode !== 'practice') return { result: `${text}\n\nIn this pane: call suggest {decision, why}; the player presses y to send it.` }
      return { result: text }
    } catch (err) {
      return { result: failureText(err), isError: true }
    }
  })

  on('tool.call', { tool: 'mcp__arena__suggest' }, async ($, e) => {
    const input = e as unknown as { decision?: unknown; why?: unknown }
    let d = input.decision
    if (typeof d === 'string') try { d = JSON.parse(d) } catch {}
    if (!d || typeof d !== 'object' || typeof (d as { kind?: unknown }).kind !== 'string') return { result: 'decision must be an object with a "kind", like {"kind":"buy","slot":0}.', isError: true }
    await refresh($)
    const r = await read($, run)
    if (!r || r.phase === 'over') return { result: 'No run in the pane to suggest for: ask the player to open one with /arena.', isError: true }
    await update($, suggestion, () => ({ runId: r.runId, decision: d as Suggestion['decision'], why: String(input.why ?? '').slice(0, 120) }))
    void open($)
    const w = await worldOf($)
    return { result: `Shown in the pane: "${decisionText(d as never, r, w.rules, w.content, { lang: await read($, lang) })}". Nothing was moved: the player presses y to send it.` }
  })

  on('ui.render', { component: 'Pane', requestId: PANE }, async ($, e) => {
    const { Box, Button, Text } = $.ui.resolve(e)
    const r = await read($, run)
    const v = await read($, view)
    const l = await read($, lang)
    const pend = await read($, pending)
    const sug = await read($, suggestion)
    const said = await read($, status)
    const cols = Math.max(20, e.props.bodyColumns)
    let w: { content: any; rules: any } | null = null
    let err = ''
    try {
      w = await worldOf($)
    } catch (x) {
      err = say(x)
    }
    const keys = keysFor(r, w?.rules ?? null, pend, sug, v, { lang: l })
    const shownKeys = new Set(keys.map(k => k.key))
    const keyRow = (
      <Box key="keys" flexWrap="wrap" columnGap={2}>
        {keys.map(k => (
          <Button key={k.key} label={k.label} hotkey={k.key} plain onPress={() => void onKey($, k.key)} />
        ))}
      </Box>
    )
    // Digits and letters no Button shows stay in the pane (a stray key would go to the prompt).
    const held = (
      <Box key="held" display="none">
        {heldKeys(keys)
          .filter(k => !shownKeys.has(k))
          .map(k => (
            <Button key={k} label={k} hotkey={k} plain onPress={() => void onKey($, k)} />
          ))}
      </Box>
    )
    const lines: { text: string; bold?: boolean; dim?: boolean; color?: string }[] = []
    if (v === 'battle') {
      const b = await read($, battle)
      try {
        const rec = b && (await battleOf($, b.id))
        if (rec) {
          const you = r && rec.player.id === r.player.id ? 'A' : r && rec.opponent.id === r.player.id ? 'B' : undefined
          const rows = Math.max(4, e.props.scroll.bodyRows - 3)
          const pg = battlePage(battleText(rec, { lang: l, view: 'steps', ...(you ? { you } : {}) }), b.page, rows)
          lines.push(...pg.lines.map((text, i) => ({ text, bold: i === 0 && pg.page === 0 })))
          if (pg.pages > 1) lines.push({ text: `${pg.page + 1}/${pg.pages}`, dim: true })
        }
      } catch (x) {
        lines.push({ text: say(x), color: 'red' })
      }
    } else if (r && w) {
      const all = paneLines(r, w.content, w.rules, { lang: l }, cols >= 100)
      lines.push({ text: all[0]!, bold: true }, ...all.slice(1).map(text => ({ text })))
      if (sug && sug.runId === r.runId) lines.push({ text: `${l === 'ru' ? 'Claude предлагает' : 'Claude suggests'}: ${decisionText(sug.decision, r, w.rules, w.content, { lang: l })}${sug.why ? ` (${sug.why})` : ''}   y ${l === 'ru' ? 'принять' : 'accept'}`, color: 'cyan' })
      if (pend) lines.push({ text: `${pend.kind.replace('2', '')}${'first' in pend ? ` ${pend.first + 1} →` : ''} …`, color: 'yellow' })
    } else lines.push({ text: 'Arena', bold: true })
    if (said) lines.push({ text: said, dim: !said.startsWith('Not') && !said.includes('answered'), color: said.includes('answered') || said.startsWith('Not') ? 'red' : undefined })
    if (err) lines.push({ text: err, color: 'red' })
    return (
      <Box flexDirection="column">
        {lines.map((x, i) => (
          <Text key={`l-${i}`} wrap="truncate" bold={x.bold} dimColor={x.dim} color={x.color}>
            {x.text}
          </Text>
        ))}
        {keyRow}
        {held}
      </Box>
    )
  })
}
