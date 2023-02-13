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
                if !resources.down_keys.is_empty() {
                    self.transition = GameState::Shop;
                }
            }
            GameState::Battle => {
                if resources.down_keys.contains(&geng::Key::R) {
                    resources.cassette.clear();
                    BattleSystem::run_battle(world, resources);
                }
            }
            GameState::Shop => {
                if resources.down_keys.contains(&geng::Key::Space) {
                    self.transition = GameState::Battle;
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
        resources: &Resources,
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
        resources: &Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        if let Some(widgets) = self
            .systems
            .get_mut(&self.current)
            .and_then(|systems| {
                Some(
                    systems
                        .iter_mut()
                        .map(|system| system.ui(cx, resources))
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
            GameState::Shop => {}
            GameState::Battle => {}
        }

        //transition to
        match self.transition {
            GameState::MainMenu => {}
            GameState::Battle => {
                BattleSystem::run_battle(world, resources);
            }
            GameState::Shop => {
                ShopSystem::refresh(world, resources);
            }
        }

        self.current = self.transition.clone();
    }
}
