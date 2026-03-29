pub mod module_bindings;
pub mod plugins;
pub mod resources;

#[cfg(test)]
mod tests {
    use crate::resources::game_state::GameState;

    #[test]
    fn game_state_default_is_title() {
        assert_eq!(GameState::default(), GameState::Title);
    }

    #[test]
    fn game_states_are_distinct() {
        let states = [
            GameState::Title,
            GameState::Login,
            GameState::Home,
            GameState::Shop,
            GameState::Battle,
            GameState::Create,
            GameState::Incubator,
        ];
        for (i, a) in states.iter().enumerate() {
            for (j, b) in states.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn tier_color_valid_range() {
        use crate::plugins::ui::tier_color;
        for tier in 0..=10 {
            let _ = tier_color(tier);
        }
    }

    #[test]
    fn rating_color_variants() {
        use crate::plugins::ui::{colors, rating_color};
        assert_eq!(rating_color(5), colors::RATING_POSITIVE);
        assert_eq!(rating_color(-3), colors::RATING_NEGATIVE);
        assert_eq!(rating_color(0), bevy_egui::egui::Color32::GRAY);
    }

    #[test]
    fn mock_content_initializes_empty() {
        use crate::plugins::collection::GameContent;
        let content = GameContent::default();
        assert!(content.abilities.is_empty());
        assert!(content.units.is_empty());
    }
}
