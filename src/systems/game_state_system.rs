use super::*;

pub struct GameStateSystem {
    pub current: GameState,
    pub transition: GameState,
    pub systems: HashMap<GameState, Vec<Box<dyn System>>>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    MainMenu,
    Shop,
    Battle,
}

impl System for GameStateSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        match self.current {
            GameState::MainMenu => {
                if resources.down_key.is_some() {
                    self.transition = GameState::Battle;
                }
            }
            GameState::Battle => {
                if resources.down_key == Some(geng::Key::R) {
                    resources.cassette.clear();
                    BattleSystem::run_battle(world, resources);
                }
            }
            GameState::Shop => {}
        }
        self.systems.get_mut(&self.current).and_then(|systems| {
            Some(
                systems
                    .iter_mut()
                    .for_each(|system| system.update(world, resources)),
            )
        });

        self.transition(world, resources);
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.systems.get(&self.current).and_then(|systems| {
            Some(
                systems
                    .iter()
                    .for_each(|system| system.draw(world, resources, framebuffer)),
            )
        });
    }
}

impl GameStateSystem {
    pub fn new(state: GameState) -> Self {
        Self {
            current: state.clone(),
            transition: state.clone(),
            systems: default(),
        }
    }

    pub fn add_systems(&mut self, state: GameState, value: Vec<Box<dyn System>>) {
        let mut systems = self.systems.remove(&state).unwrap_or_default();
        systems.extend(value.into_iter());
        self.systems.insert(state, systems);
    }

    fn transition(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if self.current == self.transition {
            return;
        }
        // transition from
        match self.current {
            GameState::MainMenu => {}
            GameState::Shop => {}
            GameState::Battle => {}
        }

        //transition to
        match self.transition {
            GameState::MainMenu => {}
            GameState::Battle => {
                BattleSystem::run_battle(world, resources);
            }
            GameState::Shop => {}
        }

        self.current = self.transition.clone();
    }
}
