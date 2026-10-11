import { expect, test } from 'claude-code/testing'

// The mod in the engine (#960), against a fake API on $.http.fetch: /arena
// shows the ranked run, a key is the player's move (no agent header), Claude's
// suggest only shows its pick, y sends it as the player's, and Claude's decide
// moves nothing on a ranked run. src/pane.spec.ts plays the same against a real server.
const RULES = { rounds: 12, offers: 3, lineSize: 5, benchSize: 3 }
const unit = (id: string, name: string, emoji: string) => ({ id, name, emoji, tier: 1, base: { pwr: 2, hp: 4 }, archetype: name, forms: { sleeping: [], awoken: [] } })
const CONTENT = { version: 'v1', units: [unit('nurse', 'Nurse', '💉'), unit('rose', 'Rose', '🌹'), unit('sab', 'Saboteur', '🧨')], abilities: {}, statuses: {} }
const runAt = (gold: number, offers: string[], line: string[]) => ({
  runId: 'r1',
  player: { id: 'p1', name: 'Tester' },
  contentVersion: 'v1',
  phase: 'shop',
  round: 1,
  hearts: 5,
  gold,
  wins: 0,
  losses: 0,
  line: line.map((id, i) => ({ uid: `u${i}`, unitId: id, name: CONTENT.units.find(u => u.id === id)!.name, emoji: '·', kind: 'unit', form: 'sleeping', copies: 1, stats: { pwr: 2, hp: 4 }, recipe: [] })),
  bench: [],
  offers: offers.map((unitId, slot) => ({ slot, unitId, tier: 1, cost: 3 })),
  nextOpponent: { player: { id: 'b1', name: 'bot-Esk' }, round: 1 },
  fights: [],
  day: 1,
  startedAt: '2026-10-11T00:00:00Z',
})

test('keys are the player moves; Claude only suggests on a ranked run', async ($, on) => {
  const calls: string[] = []
  let run = runAt(10, ['nurse', 'rose', 'sab'], [])
  on('http.fetch', ($, e) => {
    const method = e.init?.method ?? 'GET'
    const path = e.url.replace('https://arena.example/api/v1', '')
    const agent = e.init?.headers?.['X-Arena-Client'] ?? 'human'
    calls.push(`${agent} ${method} ${path} ${e.init?.body ?? ''}`.trim())
    let json: unknown = { error: 'nope' }
    let status = 200
    if (path === '/content') json = CONTENT
    else if (path === '/home') json = { rules: RULES, rating: { player: { id: 'p1', name: 'Tester' } }, activeRunId: 'r1', practiceRunId: null, day: {} }
    else if (path === '/runs/r1' && method === 'GET') json = run
    else if (path === '/runs/r1/decisions' && agent !== 'human') (status = 403), (json = { error: 'agents play practice runs only' })
    else if (path === '/runs/r1/decisions') {
      const d = JSON.parse(e.init?.body ?? '{}')
      if (d.kind === 'buy') run = runAt(run.gold - 3, run.offers.filter(o => o.slot !== d.slot).map(o => o.unitId), [...run.line.map(u => u.unitId), run.offers[d.slot]!.unitId])
      if (d.kind === 'reroll') run = runAt(run.gold - 1, ['sab', 'sab', 'sab'], run.line.map(u => u.unitId))
      json = { run }
    } else status = 404
    return { value: { status, ok: status < 300, headers: {}, text: JSON.stringify(json) } }
  })
  on('env.get', ($, e) => ({ value: e.name === 'ARENA_URL' ? 'https://arena.example' : e.name === 'ARENA_TOKEN' ? 'tok' : undefined }))
  const store = new Map<string, unknown>()
  on('store.get', ($, e) => ({ value: store.get(e.key) }))
  on('store.set', ($, e) => (store.set(e.key, e.value), { value: undefined }))
  on('command.register', ($, e) => ({ value: { command: e.name } }))
  on('tool.register', ($, e) => ({ value: { tool: `mcp__arena__${e.name}` } }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('session.start', () => ({ cwd: '/x' }))
  await $.session.start({ source: 'startup', cwd: '/x', surface: 'terminal' } as never)
  const opened = (await $.command.run({ command: 'arena', args: '' } as never)) as { text?: string }
  expect(opened.text).toContain('ranked run r1')

  const ui = await $.ui.mount({
    plugin: 'arena',
    surface: 'terminal',
    component: 'Pane',
    requestId: 'arena',
    props: { title: 'Arena', isFocused: true, bodyColumns: 80, placement: 'dock', scroll: { bodyRows: 30 } } as never,
  })
  expect(await ui.find({ type: 'Text', text: /^Arena · Round 1\/12 · ♥5 · 10g · 0W 0L · vs @bot-Esk {3}\[ranked\]$/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: /^SHOP {3}1 Nurse T1 3g {3}2 Rose T1 3g {3}3 Saboteur T1 3g$/ })).toBeDefined()

  // A key: the player's own move, no agent header.
  await ui.press({ key: '2' })
  expect(calls).toContain('human POST /runs/r1/decisions {"kind":"buy","slot":1}')
  expect(await ui.find({ type: 'Text', text: /^LINE {3}1 · Rose 2\/4 1\/3 · 2 — / })).toBeDefined()

  // Claude's decide on the ranked run reads it as an agent and moves nothing.
  calls.length = 0
  const d = (await $.tool.call({ tool: 'mcp__arena__decide', decision: { kind: 'reroll' } } as never)) as { result?: unknown }
  expect(String(d.result)).toMatch(/ranked run/)
  expect(String(d.result)).toMatch(/suggest/)
  expect(calls.filter(c => c.includes('POST'))).toEqual([])
  expect(calls.every(c => c.startsWith('agent/claude-code') || c.startsWith('human GET'))).toBe(true)

  // Its suggestion shows in the pane, still moves nothing; y sends it as the player's.
  const s = (await $.tool.call({ tool: 'mcp__arena__suggest', decision: { kind: 'reroll' }, why: 'no pair here' } as never)) as { result?: unknown }
  expect(String(s.result)).toMatch(/Nothing was moved/)
  expect(calls.filter(c => c.includes('POST'))).toEqual([])
  expect(await ui.find({ type: 'Text', text: /Claude suggests: reroll \(no pair here\) {3}y accept/ })).toBeDefined()
  await ui.press({ key: 'y' })
  expect(calls).toContain('human POST /runs/r1/decisions {"kind":"reroll"}')
  expect(await ui.find({ type: 'Text', text: /^SHOP {3}1 Saboteur/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: /Claude suggests/ })).toBeUndefined()

  // RU/EN.
  await ui.press({ key: 't' })
  expect(await ui.find({ type: 'Text', text: /^Arena · Раунд 1\/12 .*\[рейтинг\]$/ })).toBeDefined()
})
