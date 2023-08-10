use super::*;

pub struct MainMenuSystem;

impl MainMenuSystem {
    pub fn enter(from: GameState, resources: &mut Resources) {
        PanelsSystem::close_stats(resources);
        Sounds::play_sound(SoundType::Click, resources);
        fn new_run_handler(
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
                        SaveSystem::load_ladder(resources);
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
            shader.set_active(SaveSystem::have_saved_data());
        }
        fn resume_game_pre_update(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            _: &mut legion::World,
            _: &mut Resources,
        ) {
            shader.set_active(SaveSystem::have_saved_data());
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
                        SaveSystem::load_game(world, resources);
                        SaveSystem::load_ladder(resources);
                    }
                }
                _ => {}
            }
        }
        fn clear_save_pre_update(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            _: &mut legion::World,
            _: &mut Resources,
        ) {
            shader.set_active(SaveSystem::have_saved_data());
        }
        fn clear_save_handler(
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
                        SaveSystem::clear_save();
                    }
                }
                _ => {}
            }
        }

        if SaveSystem::have_saved_game() {
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
        }

        let ladder_count = SaveSystem::load_data()
            .and_then(|x| Ok(x.ladder.len()))
            .unwrap_or(0);

        let entity = new_entity();
        let uniforms = resources
            .options
            .uniforms
            .main_menu_button
            .clone()
            .insert_int("u_index".to_owned(), 0);
        Widget::Button {
            text: format!("New Run ({ladder_count})"),
            color: None,
            input_handler: new_run_handler,
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

        let entity = new_entity();
        let uniforms = resources
            .options
            .uniforms
            .main_menu_button
            .clone()
            .insert_int("u_index".to_owned(), 1);
        Widget::Button {
            text: "Clear Save".to_owned(),
            color: None,
            input_handler: clear_save_handler,
            update_handler: None,
            pre_update_handler: Some(clear_save_pre_update),
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
