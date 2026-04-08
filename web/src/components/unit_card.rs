use dioxus::prelude::*;

use crate::state::UnitSnapshot;

#[component]
pub fn UnitCard(snapshot: UnitSnapshot, is_active: bool) -> Element {
    let alive_class = if !snapshot.alive { " dead" } else { "" };
    let active_class = if is_active { " active" } else { "" };
    let class = format!("unit-card{active_class}{alive_class}");

    let hp_pct = if snapshot.base_hp > 0 {
        (snapshot.current_hp as f32 / snapshot.base_hp as f32 * 100.0).max(0.0)
    } else {
        0.0
    };

    rsx! {
        div { class,
            canvas {
                class: "unit-canvas",
                width: "80",
                height: "80",
                // Canvas rendering will be added later
            }
            div { class: "name", "{snapshot.name}" }
            div { class: "stats",
                span {
                    class: "stat-hp",
                    title: "HP: {snapshot.current_hp}/{snapshot.base_hp}",
                    "{snapshot.current_hp}/{snapshot.base_hp} HP"
                }
                span {
                    class: "stat-pwr",
                    title: "Power: {snapshot.current_pwr}",
                    "{snapshot.current_pwr} PWR"
                }
            }
            // HP bar
            div {
                style: "margin-top:4px;height:3px;background:#333;border-radius:2px;overflow:hidden",
                div {
                    style: "height:100%;background:#4ecca3;width:{hp_pct}%;transition:width 0.3s",
                }
            }
        }
    }
}
