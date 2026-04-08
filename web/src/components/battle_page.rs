use dioxus::prelude::*;

use crate::state::BattlePlaybackState;
use super::action_log::ActionLog;
use super::controls::Controls;
use super::unit_card::UnitCard;

#[component]
pub fn BattlePage(playback: Signal<BattlePlaybackState>) -> Element {
    let state = playback.read();
    let snaps = state.snapshots();
    let (left_team, right_team) = state.teams(&snaps);
    let (left_active, right_active) = state.active_fighters(&snaps);
    let is_finished = state.is_finished();
    let winner = state.result.winner;
    drop(state);

    rsx! {
        div { class: "battle-page",
            // Winner banner
            div {
                class: if is_finished { "winner-banner visible" } else { "winner-banner" },
                "Winner: {winner:?} Team!"
            }

            // Enemy team (right side)
            div { class: "team-row enemy",
                for unit in right_team.iter() {
                    UnitCard {
                        key: "{unit.id}",
                        snapshot: unit.clone(),
                        is_active: right_active == Some(unit.id),
                    }
                }
            }

            // Duel area
            {
                let snaps2 = playback.read().snapshots();
                let left_fighter = left_active.and_then(|id| snaps2.get(&id).cloned());
                let right_fighter = right_active.and_then(|id| snaps2.get(&id).cloned());
                rsx! {
                    div { class: "duel-area",
                        if let Some(lf) = left_fighter {
                            UnitCard { snapshot: lf, is_active: true }
                        }
                        div { class: "vs", "VS" }
                        if let Some(rf) = right_fighter {
                            UnitCard { snapshot: rf, is_active: true }
                        }
                    }
                }
            }

            // Ally team (left side)
            div { class: "team-row ally",
                for unit in left_team.iter() {
                    UnitCard {
                        key: "{unit.id}",
                        snapshot: unit.clone(),
                        is_active: left_active == Some(unit.id),
                    }
                }
            }

            // Action log
            ActionLog { playback }

            // Controls
            Controls { playback }
        }
    }
}
