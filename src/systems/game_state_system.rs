use super::*;

pub struct GameStateSystem {
    pub systems: HashMap<GameState, Vec<Box<dyn System>>>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum GameState {
    MainMenu,
    Shop,
    Battle,
    Gallery,
    GameOver,
    CustomGame,
}

impl System for GameStateSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        match resources.current_state {
            GameState::MainMenu => {
                if resources.options.custom_game.enable {
                    resources.transition_state = GameState::CustomGame;
                } else {
                    resources.transition_state = GameState::Shop;
                }
                // if !resources.down_keys.is_empty() {
                //     self.transition = GameState::Gallery;
                // }
            }
            GameState::Battle => {
                if resources.input.down_keys.contains(&geng::Key::R) {
                    resources.cassette.clear();
                    BattleSystem::init_battle(world, resources);
                    BattleSystem::run_battle(world, resources);
                }
                if resources.cassette.head > resources.cassette.length() + 2.0 {
                    BattleSystem::finish_battle(world, resources);
                }
            }
            GameState::Shop => {
                if resources.input.down_keys.contains(&geng::Key::Space) {
                    BattleSystem::init_battle(world, resources);
                    resources.transition_state = GameState::Battle;
                }

                if resources.input.down_keys.contains(&geng::Key::G) {
                    resources.transition_state = GameState::Gallery;
                }

                if resources.input.down_keys.contains(&geng::Key::O) {
                    resources.transition_state = GameState::GameOver;
                }
                if resources.input.down_keys.contains(&geng::Key::R) {
                    ShopSystem::restart(world, resources);
                }
                if resources.input.down_keys.contains(&geng::Key::C) {
                    resources.shop.money += 100;
                }
            }
            GameState::Gallery => {
                if resources.input.down_keys.contains(&geng::Key::G) {
                    resources.transition_state = GameState::Shop;
                }
            }
            GameState::GameOver => {
                if resources.input.down_keys.contains(&geng::Key::Enter) {
                    resources.transition_state = GameState::Shop;
                }
            }
            GameState::CustomGame => {
                if resources.input.down_keys.contains(&geng::Key::R) {
                    resources.cassette.clear();
                    BattleSystem::init_battle(world, resources);
                    BattleSystem::run_battle(world, resources);
                }
            }
        }
        self.systems
            .get_mut(&resources.current_state)
            .and_then(|systems| {
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
        self.systems
            .get(&resources.current_state)
            .and_then(|systems| {
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
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        if let Some(widgets) = self
            .systems
            .get_mut(&resources.current_state)
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

    fn pre_update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        self.systems
            .get_mut(&resources.current_state)
            .and_then(|systems| {
                Some(
                    systems
                        .iter_mut()
                        .for_each(|system| system.pre_update(world, resources)),
                )
            });
    }

    fn post_update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        self.systems
            .get_mut(&resources.current_state)
            .and_then(|systems| {
                Some(
                    systems
                        .iter_mut()
                        .for_each(|system| system.post_update(world, resources)),
                )
            });
    }
}

impl GameStateSystem {
    pub fn new() -> Self {
        Self { systems: default() }
    }

    pub fn add_systems(&mut self, state: GameState, value: Vec<Box<dyn System>>) {
        let mut systems = self.systems.remove(&state).unwrap_or_default();
        systems.extend(value.into_iter());
        self.systems.insert(state, systems);
    }

    fn transition(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.current_state == resources.transition_state {
            return;
        }
        // transition from
        match resources.current_state {
            GameState::MainMenu => {
                Shop::load_pool(world, resources);
            }
            GameState::Shop => {
                resources.cassette.clear();
            }
            GameState::Battle => {
                resources.cassette.clear();
                WorldSystem::set_var(world, VarName::IsBattle, Var::Float(0.0));
                Event::BattleOver.send(world, resources);
            }
            GameState::Gallery => {
                resources.cassette.clear();
                resources.action_queue.clear();
                resources.status_pool.status_changes.clear();
                resources.camera.camera.fov = resources.options.fov;
                WorldSystem::set_var(world, VarName::FieldPosition, Var::Vec2(vec2(0.0, 0.0)));
                SlotSystem::init_world(
                    world,
                    &resources.options,
                    hashset![Faction::Shop, Faction::Team, Faction::Dark, Faction::Light,],
                );
            }
            GameState::GameOver => {
                resources.camera.camera.fov = resources.options.fov;
                resources.camera.focus = Focus::Battle;
            }
            GameState::CustomGame => {}
        }

        resources.current_state = resources.transition_state.clone();
        //transition to
        match resources.transition_state {
            GameState::MainMenu => {}
            GameState::Battle => {
                WorldSystem::set_var(world, VarName::IsBattle, Var::Float(1.0));
                CassettePlayerSystem::init_world(world, resources);
                BattleSystem::run_battle(world, resources);
                resources.camera.focus = Focus::Battle;
            }
            GameState::Shop => {
                ShopSystem::init(world, resources);
                CassettePlayerSystem::init_world(world, resources);
                resources.camera.focus = Focus::Shop;
            }
            GameState::Gallery => {
                resources.camera.focus = Focus::Battle;
                WorldSystem::set_var(world, VarName::FieldPosition, Var::Vec2(vec2(0.0, 20.0)));
                SlotSystem::clear_world(world);
            }
            GameState::GameOver => {
                resources.camera.camera.fov = resources.options.fov * 0.5;
                resources.camera.focus = Focus::Shop;
                GameOverSystem::init(world, resources);
            }
            GameState::CustomGame => {}
        }
    }
}
