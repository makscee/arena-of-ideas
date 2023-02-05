use super::*;

pub struct GameStateSystem {
    pub current: GameState,
    pub transition: GameState,
}

impl System for GameStateSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        match self.current {
            GameState::MainMenu => {
                if resources.down_key.is_some() {
                    self.transition = GameState::Game;
                }
            }
            GameState::Game => {
                if resources.down_key == Some(geng::Key::R) {
                    resources.cassette.clear();
                    BattleSystem::run_battle(world, resources);
                }
            }
        }

        self.transition(world, resources);
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match self.current {
            GameState::MainMenu => {
                ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
            }
            GameState::Game => {
                ugli::clear(framebuffer, Some(Rgba::RED), None, None);
            }
        }
    }
}

impl GameStateSystem {
    pub fn new(state: GameState) -> Self {
        Self {
            current: state.clone(),
            transition: state.clone(),
        }
    }

    fn transition(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if self.current == self.transition {
            return;
        }
        // transition from
        match self.current {
            GameState::MainMenu => {}
            GameState::Game => {}
        }

        //transition to
        match self.transition {
            GameState::MainMenu => {}
            GameState::Game => {
                BattleSystem::run_battle(world, resources);
            }
        }

        self.current = self.transition.clone();
    }
}
