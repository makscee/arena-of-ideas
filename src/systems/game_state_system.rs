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
    Gallery,
}

impl System for GameStateSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        match self.current {
            GameState::MainMenu => {
                self.transition = GameState::Shop;
                // if !resources.down_keys.is_empty() {
                //     self.transition = GameState::Gallery;
                // }
            }
            GameState::Battle => {
                if resources.down_keys.contains(&geng::Key::R) {
                    resources.cassette.clear();
                    resources.rounds.next_round -= 1;
                    BattleSystem::run_battle(world, resources);
                }
                if resources.cassette.head > resources.cassette.length() + 2.0 {
                    self.transition = GameState::Shop;
                }
            }
            GameState::Shop => {
                if resources.down_keys.contains(&geng::Key::Space) {
                    self.transition = GameState::Battle;
                }

                if resources.down_keys.contains(&geng::Key::G) {
                    self.transition = GameState::Gallery;
                }
            }
            GameState::Gallery => {
                if resources.down_keys.contains(&geng::Key::S) {
                    self.transition = GameState::Shop;
                }
            }
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
        resources: &mut Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        self.systems.get(&self.current).and_then(|systems| {
            Some(
                systems
                    .iter()
                    .for_each(|system| system.draw(world, resources, framebuffer)),
            )
        });
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        if let Some(widgets) = self
            .systems
            .get_mut(&self.current)
            .and_then(|systems| {
                Some(
                    systems
                        .iter_mut()
                        .map(|system| system.ui(cx, world, resources))
                        .collect_vec(),
                )
            })
            .and_then(|widgets| Some(widgets))
        {
            if !widgets.is_empty() {
                return Box::new(geng::ui::stack(widgets));
            }
        }
        Box::new(ui::Void)
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
            GameState::Shop => {
                resources.cassette.clear();
            }
            GameState::Battle => {
                resources.cassette.clear();
                WorldSystem::set_var(world, VarName::IsBattle, &Var::Float(0.0));
                BattleSystem::finish_battle(world, resources);
            }
            GameState::Gallery => {
                resources.cassette.clear();
                resources.camera.fov = resources.options.fov;
                WorldSystem::set_var(world, VarName::FieldPosition, &Var::Float(0.0))
            }
        }

        //transition to
        match self.transition {
            GameState::MainMenu => {}
            GameState::Battle => {
                WorldSystem::set_var(world, VarName::IsBattle, &Var::Float(1.0));
                BattleSystem::run_battle(world, resources);
            }
            GameState::Shop => {
                ShopSystem::init(world, resources);
            }
            GameState::Gallery => {
                WorldSystem::set_var(world, VarName::FieldPosition, &Var::Float(20.0))
            }
        }

        self.current = self.transition.clone();
    }
}
