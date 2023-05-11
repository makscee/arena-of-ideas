use strum_macros::Display;

use super::*;

pub struct GameStateSystem {
    pub systems: HashMap<GameState, Vec<Box<dyn System>>>,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug, Display)]
pub enum GameState {
    MainMenu,
    Shop,
    Battle,
    Gallery,
    Sacrifice,
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
                    resources.transition_state = resources.options.initial_state;
                }
            }
            GameState::Battle => {
                if resources.input_data.down_keys.contains(&R) {
                    resources.tape_player.head = 0.0;
                }
                if resources.input_data.down_keys.contains(&Escape) {
                    resources.tape_player.head = resources.tape_player.tape.length() - 3.0;
                }
                if resources.tape_player.tape.length() < resources.tape_player.head {
                    BattleSystem::finish_floor_battle(world, resources);
                }
            }
            GameState::Shop => {
                if resources.input_data.down_keys.contains(&Space) {
                    // ShopSystem::switch_to_battle(world, resources);
                    // resources.transition_state = GameState::Battle;
                    match resources.camera.focus == Focus::Shop {
                        true => ShopSystem::show_battle_choice_widget(resources),
                        false => resources.transition_state = GameState::Battle,
                    }
                }

                if resources.input_data.down_keys.contains(&G) {
                    resources.transition_state = GameState::Gallery;
                }

                if resources.input_data.down_keys.contains(&O) {
                    resources.transition_state = GameState::GameOver;
                }
                if resources.input_data.down_keys.contains(&R) {
                    resources.transition_state = GameState::MainMenu;
                }
                if resources.input_data.down_keys.contains(&C) {
                    ShopSystem::change_g(100, world);
                }
                if resources.input_data.down_keys.contains(&L) {
                    SaveSystem::load(world, resources);
                }
                if resources.input_data.down_keys.contains(&S) {
                    SaveSystem::save(world, resources);
                }
                if resources.input_data.down_keys.contains(&X) {
                    GameStateSystem::set_transition(GameState::Sacrifice, resources);
                }
                if resources.input_data.down_keys.contains(&B) {
                    BonusEffectPool::load_widget(5, world, resources);

                    // resources.tape_player.tape.push_to_queue(
                    //     NodeCluster::new(
                    //         Widget::BattleOverPanel {
                    //             score: 3,
                    //             options: &resources.options,
                    //         }
                    //         .generate_node()
                    //         .lock(NodeLockType::Empty),
                    //     ),
                    //     resources.tape_player.head,
                    // );
                }
                if resources.input_data.down_keys.contains(&P) {
                    if let Some(entity) = SlotSystem::find_unit_by_slot(1, &Faction::Shop, world) {
                        world.remove(entity);
                    }
                    resources.hero_pool.list_top().clone().unpack(
                        world,
                        resources,
                        1,
                        Some(SlotSystem::get_position(1, &Faction::Shop, resources)),
                        TeamSystem::entity(&Faction::Shop, world),
                    );
                }
            }
            GameState::Gallery => {
                if resources.input_data.down_keys.contains(&G) {
                    resources.transition_state = GameState::Shop;
                }
            }
            GameState::GameOver => {
                if resources.input_data.down_keys.contains(&Enter) {
                    resources.transition_state = GameState::Shop;
                }
            }
            GameState::CustomGame => {
                if resources.input_data.down_keys.contains(&R) {
                    resources.transition_state = GameState::MainMenu;
                    resources.tape_player.head = 0.0;
                }
            }
            GameState::Sacrifice => {
                resources.tape_player.tape.persistent_node =
                    Node::default().lock(NodeLockType::Factions {
                        factions: hashset! {Faction::Team},
                        world,
                        resources,
                    });
                if resources.input_data.down_keys.contains(&R) {
                    resources.transition_state = GameState::MainMenu;
                    resources.tape_player.head = 0.0;
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

    pub fn set_transition(to: GameState, resources: &mut Resources) {
        resources.transition_state = to;
    }

    fn change_state(to: GameState, world: &mut legion::World, resources: &mut Resources) {
        resources
            .tape_player
            .tape
            .close_all_panels(resources.tape_player.head);
        let from = resources.current_state;
        Self::leave_state(from, to, world, resources);
        Self::enter_state(from, to, world, resources);
        resources.current_state = to;
    }

    fn leave_state(
        from: GameState,
        to: GameState,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        debug!("leave_state {from} -> {to}");
        match from {
            GameState::Shop => {
                ShopSystem::leave(world, resources);
            }
            GameState::Battle => {
                resources.tape_player.clear();
                Event::BattleEnd.send(world, resources);
                Event::ShopStart.send(world, resources);
            }
            GameState::GameOver => {
                resources.camera.camera.fov = resources.options.fov;
                resources.camera.focus = Focus::Battle;
            }
            GameState::Sacrifice
            | GameState::Gallery
            | GameState::MainMenu
            | GameState::CustomGame => {}
        }
    }

    fn enter_state(
        from: GameState,
        to: GameState,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        debug!("enter_state {from} -> {to}");
        Node::new_panel_scaled(
            resources
                .options
                .shaders
                .state_indicator
                .clone()
                .set_string("u_text", to.to_string(), 1),
        )
        .lock(NodeLockType::Empty)
        .push_as_panel(new_entity(), resources);
        match to {
            GameState::MainMenu => {
                Game::restart(world, resources);
            }
            GameState::Shop => {
                if from == GameState::MainMenu {
                    ShopSystem::init_game(world, resources);
                    SlotSystem::create_entries(world, resources);
                    // BonusEffectPool::load_widget(3, world, resources);
                } else if from == GameState::Sacrifice {
                    PackedTeam::new("Dark".to_owned(), default()).unpack(
                        &Faction::Dark,
                        world,
                        resources,
                    );
                    PackedTeam::new("Light".to_owned(), default()).unpack(
                        &Faction::Light,
                        world,
                        resources,
                    );
                    SacrificeSystem::show_bonus_widget(world, resources);
                }
                ShopSystem::enter(world, resources);
                resources.camera.focus = Focus::Shop;
            }
            GameState::Battle => {
                resources.camera.focus = Focus::Battle;
                let mut tape = Some(Tape::default());
                BattleSystem::run_battle(world, resources, &mut tape);
                resources.tape_player.clear();
                resources.tape_player.tape = tape.unwrap();
            }
            GameState::Gallery => {}
            GameState::Sacrifice => {
                resources.camera.focus = Focus::Shop;

                fn input_handler(
                    event: HandleEvent,
                    _: legion::Entity,
                    _: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            SacrificeSystem::sacrifice_marked(world, resources);
                        }
                        _ => {}
                    };
                }
                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    _: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    shader.set_active(!resources.sacrifice_data.marked_units.is_empty());
                }

                Widget::Button {
                    text: "Accept".to_owned(),
                    input_handler,
                    update_handler: Some(update_handler),
                    options: &resources.options,
                    uniforms: resources.options.uniforms.ui_button.clone(),
                }
                .generate_node()
                .lock(NodeLockType::Empty)
                .push_as_panel(new_entity(), resources);
            }
            GameState::GameOver => {
                resources.camera.focus = Focus::Shop;
                GameOverSystem::init(world, resources);
            }
            GameState::CustomGame => {
                resources.camera.focus = Focus::Battle;
                let light = resources
                    .options
                    .custom_game
                    .light
                    .clone()
                    .expect("Light team not set for custom game in options.json")
                    .into();
                let dark = resources
                    .options
                    .custom_game
                    .dark
                    .clone()
                    .expect("Light team not set for custom game in options.json")
                    .into();
                BattleSystem::init_battle(&light, &dark, world, resources);
                let mut tape = Some(Tape::default());
                BattleSystem::run_battle(world, resources, &mut tape);
                let tape = tape.unwrap();
                dbg!(tape.length());
                resources.tape_player.tape = tape;
            }
        }
        SlotSystem::handle_state_enter(to, world, resources);
    }

    fn transition(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.current_state == resources.transition_state {
            return;
        }
        Self::change_state(resources.transition_state, world, resources);
    }
}
