use dioxus::prelude::*;

use crate::state::BattlePlaybackState;

#[component]
pub fn Controls(playback: Signal<BattlePlaybackState>) -> Element {
    let state = playback.read();
    let current = state.current_index;
    let total = state.total_actions();
    let at_start = current == 0;
    let at_end = state.is_finished();
    let auto_play = state.auto_play;
    let speed = state.speed;
    drop(state);

    // Auto-play via coroutine
    let _auto_player = use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(200).await;
            let mut state = playback.write();
            if state.auto_play && !state.is_finished() {
                state.step_forward();
            } else if state.is_finished() {
                state.auto_play = false;
            }
        }
    });

    rsx! {
        div { class: "controls",
            button {
                disabled: at_start,
                onclick: move |_| playback.write().reset(),
                "Reset"
            }
            button {
                disabled: at_start,
                onclick: move |_| playback.write().step_back(),
                "Step Back"
            }
            button {
                disabled: at_end,
                onclick: move |_| playback.write().step_forward(),
                "Step"
            }
            button {
                class: if auto_play { "active" } else { "" },
                onclick: move |_| {
                    let mut state = playback.write();
                    state.auto_play = !state.auto_play;
                },
                if auto_play { "Pause" } else { "Play" }
            }
            button {
                disabled: at_end,
                onclick: move |_| playback.write().jump_to_end(),
                "End"
            }
            div { class: "step-display", "{current} / {total}" }
            div { class: "speed-label",
                button {
                    onclick: move |_| {
                        let mut s = playback.write();
                        s.speed = match s.speed {
                            x if x <= 0.5 => 1.0,
                            x if x <= 1.0 => 2.0,
                            _ => 0.5,
                        };
                    },
                    "{speed}x"
                }
            }
        }
    }
}
