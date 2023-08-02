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
    Victory,
    CustomGame,
    Intro,
}

impl System for GameStateSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.input_data.down_keys.contains(&R) {
            Game::restart(world, resources);
        }
        match resources.current_state {
            GameState::Intro => {
                if resources.options.custom_game.enable {
                    resources.transition_state = GameState::CustomGame;
                } else {
                    resources.transition_state = resources.options.initial_state;
                }
            }
            GameState::MainMenu => {}
            GameState::Battle => {
                if resources.tape_player.tape.length() == 0.0 {
                    resources.tape_player.tape.persistent_node =
                        Node::default().lock(NodeLockType::Factions {
                            factions: Faction::battle(),
                            world,
                            resources,
                        });
                } else {
                    if resources.input_data.down_keys.contains(&Escape) {
                        resources.tape_player.head = resources.tape_player.tape.length() - 0.5;
                    }
                    if resources.tape_player.tape.length() > 0.0
                        && resources.tape_player.tape.length() < resources.tape_player.head
                    {
                        BattleSystem::finish_ladder_battle(world, resources);
                    }
                }
            }
            GameState::Shop => {
                if resources.input_data.down_keys.contains(&Space) {
                    match resources.camera.focus == Focus::Shop {
                        true => Ladder::start_next_battle(world, resources),
                        false => resources.transition_state = GameState::Battle,
                    }
                }

                if resources.input_data.down_keys.contains(&G) {
                    resources.transition_state = GameState::Gallery;
                }

                if resources.input_data.down_keys.contains(&O) {
                    resources.transition_state = GameState::GameOver;
                }
                if resources.input_data.down_keys.contains(&C) {
                    ShopSystem::change_g(100, Some("Cheat"), world, resources);
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
                if resources.input_data.down_keys.contains(&P) {
                    PanelsSystem::open_push(
                        resources.options.colors.secondary,
                        "Test 1",
                        "This is a\ntest push",
                        resources,
                    );
                }
            }
            GameState::Gallery => {
                if resources.input_data.down_keys.contains(&G) {
                    resources.transition_state = GameState::Shop;
                }
                if resources.input_data.down_keys.contains(&C) {
                    resources.gallery_data.card = !resources.gallery_data.card;
                }
                resources.gallery_data.cur_card += (resources.gallery_data.card as i32 as f32
                    - resources.gallery_data.cur_card)
                    * resources.delta_time
                    * 5.0;
                if let Some(entity) = resources.gallery_data.panel {
                    let value = resources.gallery_data.cur_card;
                    PanelsSystem::find_alert_mut(entity, resources)
                        .unwrap()
                        .shader
                        .insert_float_ref("u_card".to_owned(), value);
                }
            }
            GameState::GameOver | GameState::Victory => {
                if resources.input_data.down_keys.contains(&Enter) {
                    resources.transition_state = GameState::Shop;
                }
            }
            GameState::CustomGame => {}
            GameState::Sacrifice => {
                resources.tape_player.tape.persistent_node =
                    Node::default().lock(NodeLockType::Factions {
                        factions: hashset! {Faction::Team},
                        world,
                        resources,
                    });
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

    pub fn current_is(value: GameState, resources: &Resources) -> bool {
        resources.current_state == value
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
        resources.current_state = to;
        Self::enter_state(from, to, world, resources);
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
                BattleSystem::clear_world(world, resources);
                resources.tape_player.clear();
                Event::BattleEnd.send(world, resources);
                Event::ShopStart.send(world, resources);
            }
            GameState::GameOver => {
                resources.camera.camera.fov = resources.options.fov;
                resources.camera.focus = Focus::Battle;
            }
            GameState::Gallery => {
                GallerySystem::leave_state(resources);
            }
            GameState::Sacrifice
            | GameState::MainMenu
            | GameState::CustomGame
            | GameState::Victory
            | GameState::Intro => {}
        }
    }

    fn enter_state(
        from: GameState,
        to: GameState,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        debug!("enter_state {from} -> {to}");
        SlotSystem::handle_state_enter(to, world);
        match to {
            GameState::MainMenu => {
                PanelsSystem::close_stats(resources);
                fn new_solo_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    _: &mut Shader,
                    _: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if resources
                                .tape_player
                                .tape
                                .close_panels(entity, resources.tape_player.head)
                            {
                                GameStateSystem::set_transition(GameState::Shop, resources);
                                resources.ladder.levels.clear();
                            }
                        }
                        _ => {}
                    }
                }
                fn resume_solo_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    _: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            if shader.is_active()
                                && resources
                                    .tape_player
                                    .tape
                                    .close_panels(entity, resources.tape_player.head)
                            {
                                GameStateSystem::set_transition(GameState::Shop, resources);
                            }
                        }
                        _ => {}
                    }
                }
                fn resume_pre_update(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    _: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    shader.set_active(!resources.ladder.levels.is_empty());
                }

                let entity = new_entity();
                let uniforms = resources
                    .options
                    .uniforms
                    .main_menu_button
                    .clone()
                    .insert_int("u_index".to_owned(), 0);
                Widget::Button {
                    text: format!("Resume Ladder ({})", Ladder::count(resources)),
                    color: None,
                    input_handler: resume_solo_handler,
                    update_handler: None,
                    pre_update_handler: Some(resume_pre_update),
                    options: &resources.options,
                    uniforms,
                    shader: None,
                    hover_hints: default(),
                    entity,
                }
                .generate_node()
                .lock(NodeLockType::Empty)
                .push_as_panel(entity, resources);

                let entity = new_entity();
                let uniforms = resources
                    .options
                    .uniforms
                    .main_menu_button
                    .clone()
                    .insert_int("u_index".to_owned(), 1);
                Widget::Button {
                    text: "New Ladder".to_owned(),
                    color: None,
                    input_handler: new_solo_handler,
                    update_handler: None,
                    pre_update_handler: None,
                    options: &resources.options,
                    uniforms,
                    shader: None,
                    hover_hints: default(),
                    entity,
                }
                .generate_node()
                .lock(NodeLockType::Empty)
                .push_as_panel(entity, resources);
            }
            GameState::Shop => {
                PanelsSystem::close_all_alerts(resources);
                if from == GameState::MainMenu {
                    ShopSystem::init_game(world, resources);
                    SlotSystem::create_entries(world, resources);
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
                }
                ShopSystem::enter(from, world, resources);
                PanelsSystem::open_stats(world, resources);
                resources.camera.focus = Focus::Shop;
            }
            GameState::Battle => {
                BattleSystem::enter_state(world, resources);
            }
            GameState::Gallery => {
                GallerySystem::enter_state(resources);
            }
            GameState::Sacrifice => {
                if from == GameState::MainMenu {
                    ShopSystem::init_game(world, resources);
                    SlotSystem::create_entries(world, resources);
                    let team = TeamSystem::entity(Faction::Team, world);
                    HeroPool::find_by_name("Berserker", &resources)
                        .unwrap()
                        .clone()
                        .unpack(world, resources, 1, None, team);
                }
                SacrificeSystem::enter_state(world, resources);
            }
            GameState::GameOver => {
                resources.camera.focus = Focus::Shop;
                GameOverSystem::init(world, resources);
            }
            GameState::Victory => {
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
                let result = BattleSystem::run_battle(world, resources, &mut tape);
                let tape = tape.unwrap();
                dbg!(tape.length());
                debug!("Battle result: {result}");
                resources.tape_player.tape = tape;
            }
            GameState::Intro => {}
        }
    }

    fn transition(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.current_state == resources.transition_state {
            return;
        }
        Self::change_state(resources.transition_state, world, resources);
    }
}
