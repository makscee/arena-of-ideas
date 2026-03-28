use bevy::prelude::*;

use crate::resources::game_state::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Update, handle_title_input.run_if(in_state(GameState::Title)))
            .add_systems(OnEnter(GameState::Title), setup_title)
            .add_systems(OnExit(GameState::Title), cleanup::<TitleScreen>)
            .add_systems(OnEnter(GameState::Login), setup_login)
            .add_systems(OnExit(GameState::Login), cleanup::<LoginScreen>);
    }
}

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
struct LoginScreen;

fn setup_title(mut commands: Commands) {
    commands.spawn((
        TitleScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Arena of Ideas"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new("Press SPACE to start"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
        ));
    });
}

fn handle_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Home);
    }
}

fn setup_login(mut commands: Commands) {
    commands.spawn((
        LoginScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Connecting..."),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
