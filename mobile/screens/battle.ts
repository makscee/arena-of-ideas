// The battle viewer (mission #574). Slice 9 owns this screen: playback with
// captions, tap any change to trace its chain, "why I lost", speed 1×/2×/skip.
// The shop shows it after every fight and goes on to the result when it calls
// onDone. Give the skip control data-testid="battle-skip": the phone e2e taps
// it to get through a fight.
import type { BattleRecord, FightResult, RunView } from "../../src/mvp/contract";

export function battleScreen(a: { run: RunView; fight: FightResult; battle: BattleRecord; onDone: () => void }): void {
  // Slice 9 replaces this stub, which goes straight to the result.
  a.onDone();
}
