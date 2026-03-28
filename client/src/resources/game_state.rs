use bevy::prelude::*;

/// Top-level game state machine.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Title,
    Login,
    Home,
    Shop,
    Battle,
    Create,
    Incubator,
}
