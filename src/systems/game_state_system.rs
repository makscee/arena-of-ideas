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
                let units = <&UnitComponent>::query().iter(world).collect_vec();
                if units.len() > 1 && resources.action_queue.is_empty() {
                    let context = Context {
                        owner: units[0].entity,
                        target: units[1].entity,
                        creator: units[0].entity,
                        vars: default(),
                        status: default(),
                    };
                    resources
                        .action_queue
                        .push_back(Action::new(context, Effect::Damage { value: 1 }));
                    let context = Context {
                        owner: units[1].entity,
                        target: units[0].entity,
                        creator: units[1].entity,
                        vars: default(),
                        status: default(),
                    };
                    resources
                        .action_queue
                        .push_back(Action::new(context, Effect::Damage { value: 1 }));
                }

                let dead_units = <(&UnitComponent, &HpComponent, &Context)>::query()
                    .iter(world)
                    .filter_map(|(unit, hp, context)| {
                        match hp
                            .current
                            .get(&context.vars)
                            .expect("Cant find hp for unit")
                            <= 0
                        {
                            true => Some(unit.entity),
                            false => None,
                        }
                    })
                    .collect_vec();
                if !dead_units.is_empty() {
                    dbg!(dead_units.clone());
                    dead_units.iter().for_each(|entity| {
                        world.remove(*entity);
                    });
                }
            }
        }

        self.transition();
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match self.current {
            GameState::MainMenu => {
                ugli::clear(framebuffer, Some(Rgba::BLUE), None, None);
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

    fn transition(&mut self) {
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
            GameState::Game => {}
        }

        self.current = self.transition.clone();
    }
}
