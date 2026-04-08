mod components;
mod state;
mod test_data;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    let battle = test_data::sample_battle();
    let units = test_data::sample_units();
    let playback = use_signal(|| state::BattlePlaybackState::new(battle, units));

    rsx! {
        components::BattlePage { playback }
    }
}
