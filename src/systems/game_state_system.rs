use super::*;

pub struct GameStateSystem {}

impl GameStateSystem {
    pub fn update(world: &mut World, _delta_time: Time) {
        let state = Self::get_state(world);

        match state {
            GameState::MainMenu => {}
            GameState::Game => {}
        }
    }

    pub fn draw(world: &World, framebuffer: &mut ugli::Framebuffer) {
        let state = Self::get_state(world);
        match state {
            GameState::MainMenu => {
                ugli::clear(framebuffer, Some(Rgba::GRAY), None, None);
            }
            GameState::Game => {
                ugli::clear(framebuffer, Some(Rgba::RED), None, None);
            }
        }
    }

    pub fn handle_event(world: &mut World, event: Event) {
        let state = Self::get_mut_state(world);
        match state {
            GameState::MainMenu => match event {
                Event::KeyDown { key: _ } => {
                    *state = GameState::Game;
                }
                _ => {}
            },
            GameState::Game => {}
        }
    }

    fn get_state(world: &World) -> &GameState {
        let mut query = <&GameState>::query();
        query.iter(world).next().expect("No state found in world")
    }

    fn get_mut_state(world: &mut World) -> &mut GameState {
        <&mut GameState>::query()
            .iter_mut(world)
            .next()
            .expect("No state found in world")
    }
}
