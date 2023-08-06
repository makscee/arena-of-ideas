use super::*;

pub struct TapePlayerSystem;

impl TapePlayerSystem {
    pub fn new() -> Self {
        Self {}
    }
}

const REWIND_SPEED: Time = 4.0;
const VELOCITY_CORRECTION_SPEED: Time = 4.0;

impl System for TapePlayerSystem {
    fn update(&mut self, _: &mut legion::World, resources: &mut Resources) {
        resources.tape_player.head += resources.tape_player.velocity * resources.delta_time;
        let mut need_velocity = match resources.tape_player.mode {
            TapePlayMode::Play => 1.0,
            TapePlayMode::Stop => (resources.global_time * 4.0).sin() * 0.3,
        } + resources.tape_player.need_velocity;
        if resources.input_data.pressed_keys.contains(&Left) {
            need_velocity -= REWIND_SPEED;
        } else if resources.input_data.pressed_keys.contains(&Right) {
            need_velocity += REWIND_SPEED;
        }
        if resources.input_data.down_keys.contains(&Space) {
            Self::press_play(resources);
        }
        resources.tape_player.velocity += (need_velocity - resources.tape_player.velocity)
            * resources.delta_time
            * VELOCITY_CORRECTION_SPEED;
        resources.tape_player.need_velocity = 0.0;
    }

    fn draw(&self, _: &legion::World, resources: &mut Resources, _: &mut ugli::Framebuffer) {
        if resources.battle_data.tape_indicator_entity.is_none() {
            resources.battle_data.tape_indicator_entity = Some(new_entity());
        }
        if resources.battle_data.tape_forward_entity.is_none() {
            resources.battle_data.tape_forward_entity = Some(new_entity());
        }
        if resources.battle_data.tape_backward_entity.is_none() {
            resources.battle_data.tape_backward_entity = Some(new_entity());
        }

        let mut shader = resources
            .options
            .shaders
            .tape_indicator
            .clone()
            .merge_uniforms(
                &hashmap! {
                    "u_head" => ShaderUniform::Float(resources.tape_player.head),
                    "u_velocity" => ShaderUniform::Float(resources.tape_player.velocity),
                }
                .into(),
                true,
            );
        ButtonSystem::add_button_handlers(&mut shader.middle);
        fn click_handler(
            event: HandleEvent,
            entity: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    debug!("Click Play");
                    TapePlayerSystem::press_play(resources);
                }
                _ => {}
            }
        }
        shader.middle.input_handlers.push(click_handler);
        shader.middle.entity = resources.battle_data.tape_indicator_entity;

        fn rewind_handler(
            event: HandleEvent,
            entity: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Press => {
                    if Some(entity) == resources.battle_data.tape_forward_entity {
                        TapePlayerSystem::rewind(1.0, resources)
                    } else if Some(entity) == resources.battle_data.tape_backward_entity {
                        TapePlayerSystem::rewind(-1.0, resources)
                    }
                }
                _ => {}
            }
        }
        let mut forward = resources
            .options
            .shaders
            .tape_rewind_button
            .clone()
            .insert_float("u_direction".to_owned(), 1.0);
        ButtonSystem::add_button_handlers(&mut forward.middle);
        forward.middle.entity = resources.battle_data.tape_forward_entity;
        forward.middle.input_handlers.push(rewind_handler);
        let mut backward = resources
            .options
            .shaders
            .tape_rewind_button
            .clone()
            .insert_float("u_direction".to_owned(), -1.0);
        ButtonSystem::add_button_handlers(&mut backward.middle);
        backward.middle.entity = resources.battle_data.tape_backward_entity;
        backward.middle.input_handlers.push(rewind_handler);

        shader.after.push(forward);
        shader.after.push(backward);

        resources.frame_shaders.push(shader);
    }
}

impl TapePlayerSystem {
    pub fn get_shaders(
        entity_shaders: HashMap<legion::Entity, ShaderChain>,
        resources: &mut Resources,
    ) -> Vec<ShaderChain> {
        let ts = resources.tape_player.head;
        let mut tape = Tape::default();
        mem::swap(&mut tape, &mut resources.tape_player.tape);
        let shaders = tape.get_shaders(ts, entity_shaders, resources);
        mem::swap(&mut tape, &mut resources.tape_player.tape);
        shaders
    }

    pub fn press_play(resources: &mut Resources) {
        if resources.current_state == GameState::Battle
            || resources.current_state == GameState::CustomGame
        {
            let player = &mut resources.tape_player;
            player.mode = match player.mode {
                TapePlayMode::Play => TapePlayMode::Stop,
                TapePlayMode::Stop => TapePlayMode::Play,
            }
        }
    }

    pub fn rewind(direction: f32, resources: &mut Resources) {
        if direction < 0.0 {
            resources.tape_player.need_velocity -= REWIND_SPEED;
        } else if direction > 0.0 {
            resources.tape_player.need_velocity += REWIND_SPEED;
        }
    }
}
