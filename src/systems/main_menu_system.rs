use super::*;

pub struct MainMenuSystem;

impl MainMenuSystem {
    pub fn enter(from: GameState, resources: &mut Resources) {
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
        fn resume_game_pre_update(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            _: &mut legion::World,
            _: &mut Resources,
        ) {
            shader.set_active(SaveSystem::have_saved_game());
        }
        fn resume_game_handler(
            event: HandleEvent,
            entity: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    if resources
                        .tape_player
                        .tape
                        .close_panels(entity, resources.tape_player.head)
                    {
                        SaveSystem::load(world, resources);
                    }
                }
                _ => {}
            }
        }

        let entity = new_entity();
        let uniforms = resources
            .options
            .uniforms
            .main_menu_button
            .clone()
            .insert_int("u_index".to_owned(), -1);
        Widget::Button {
            text: "Resume Game".to_owned(),
            color: None,
            input_handler: resume_game_handler,
            update_handler: None,
            pre_update_handler: Some(resume_game_pre_update),
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
}
