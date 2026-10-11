// The Arena mod's state (#960). The run is the API's RunView
// (src/mvp/contract.ts), held here by the fields the mod reads.
export type Lang = 'en' | 'ru'
export type View = 'run' | 'battle'
export type Decision = { kind: string; [field: string]: unknown }
export type RunView = {
  runId: string
  mode?: 'ranked' | 'practice'
  phase: 'shop' | 'crown' | 'over'
  player: { id: string; name: string }
  fights: { battleId: string }[]
  [field: string]: unknown
}
export type Pending = { kind: 'sell' | 'lock' | 'move' | 'fuse' } | { kind: 'move2' | 'fuse2'; first: number }
export type Suggestion = { runId: string; decision: Decision; why: string }

declare module 'claude-code' {
  interface PluginState {
    arena: {
      run: RunView | null
      status: string
      pending: Pending | null
      suggestion: Suggestion | null
      view: View
      battle: { id: string; page: number } | null
      lang: Lang
      busy: boolean
    }
  }
}
