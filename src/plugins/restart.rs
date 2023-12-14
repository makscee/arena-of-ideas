use super::*;

pub struct RestartPlugin;

impl Plugin for RestartPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Restart), Self::restart);
    }
}

impl RestartPlugin {
    #[allow(clippy::type_complexity)]
    fn restart(
        query: Query<Entity, Or<(&Unit, &Corpse, &Representation, &VarState, &Status)>>,
        mut commands: Commands,
        mut state: ResMut<NextState<GameState>>,
        mut time: ResMut<Time>,
        mut game_timer: ResMut<GameTimer>,
    ) {
        for unit in query.iter() {
            commands.entity(unit).despawn_recursive();
        }
        *time = Time::new(Instant::now());
        game_timer.reset();
        state.set(GameState::MainMenu);
    }
}
