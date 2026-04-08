use dioxus::prelude::*;
use shared::battle::{BattleAction, StatKind};

use crate::state::BattlePlaybackState;

#[component]
pub fn ActionLog(playback: Signal<BattlePlaybackState>) -> Element {
    let state = playback.read();
    let visible = state.visible_actions();

    rsx! {
        div { class: "action-log",
            if visible.is_empty() {
                div { class: "turn-header", "Press Step or Play to begin..." }
            }
            for (_idx, action) in visible.iter().rev() {
                {render_action_card(action, &state)}
            }
        }
    }
}

fn render_action_card(action: &BattleAction, state: &BattlePlaybackState) -> Element {
    match action {
        BattleAction::Spawn { unit, side, .. } => {
            let name = state.unit_name(*unit);
            rsx! {
                div { class: "action-card spawn",
                    span { class: "source", "{name}" }
                    " enters the battle ({side:?})"
                }
            }
        }
        BattleAction::Damage {
            source,
            target,
            amount,
        } => {
            let src = state.unit_name(*source);
            let tgt = state.unit_name(*target);
            rsx! {
                div { class: "action-card damage",
                    span { class: "source", "{src}" }
                    " deals "
                    span { class: "amount-dmg", "{amount}" }
                    " damage to "
                    span { class: "target", "{tgt}" }
                }
            }
        }
        BattleAction::Heal {
            source,
            target,
            amount,
        } => {
            let src = state.unit_name(*source);
            let tgt = state.unit_name(*target);
            rsx! {
                div { class: "action-card heal",
                    span { class: "source", "{src}" }
                    " heals "
                    span { class: "target", "{tgt}" }
                    " for "
                    span { class: "amount-heal", "{amount}" }
                }
            }
        }
        BattleAction::Death { unit } => {
            let name = state.unit_name(*unit);
            rsx! {
                div { class: "action-card death",
                    span { class: "target", "{name}" }
                    " has been slain!"
                }
            }
        }
        BattleAction::AbilityUsed {
            source,
            ability_name,
        } => {
            let name = state.unit_name(*source);
            rsx! {
                div { class: "action-card ability",
                    span { class: "source", "{name}" }
                    " uses "
                    span { class: "ability-name", "{ability_name}" }
                }
            }
        }
        BattleAction::StatChange { unit, stat, delta } => {
            let name = state.unit_name(*unit);
            let stat_name = match stat {
                StatKind::Hp => "HP",
                StatKind::Pwr => "PWR",
                StatKind::Dmg => "DMG",
            };
            let sign = if *delta > 0 { "+" } else { "" };
            rsx! {
                div { class: "action-card stat-change",
                    span { class: "source", "{name}" }
                    " {stat_name} {sign}{delta}"
                }
            }
        }
        BattleAction::Fatigue { amount } => {
            rsx! {
                div { class: "action-card fatigue",
                    "Fatigue deals "
                    span { class: "amount-dmg", "{amount}" }
                    " to all units"
                }
            }
        }
        _ => rsx! {},
    }
}
