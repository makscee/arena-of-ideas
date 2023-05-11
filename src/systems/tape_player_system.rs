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
        let player = &mut resources.tape_player;
        let head = &mut player.head;
        *head += player.velocity * resources.delta_time;
        let mut need_velocity = match player.mode {
            TapePlayMode::Play => 1.0,
            TapePlayMode::Stop => 0.0,
        };
        if resources.input_data.pressed_keys.contains(&Left) {
            need_velocity = -REWIND_SPEED;
        } else if resources.input_data.pressed_keys.contains(&Right) {
            need_velocity = REWIND_SPEED;
        }
        if resources.input_data.down_keys.contains(&Space)
            && (resources.current_state == GameState::Battle
                || resources.current_state == GameState::CustomGame)
        {
            player.mode = match player.mode {
                TapePlayMode::Play => TapePlayMode::Stop,
                TapePlayMode::Stop => TapePlayMode::Play,
            }
        }
        player.velocity +=
            (need_velocity - player.velocity) * resources.delta_time * VELOCITY_CORRECTION_SPEED;
    }

    fn draw(&self, _: &legion::World, resources: &mut Resources, _: &mut ugli::Framebuffer) {
        let shader = resources
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
        resources.frame_shaders.push(shader);
    }
}

impl TapePlayerSystem {
    pub fn get_shaders(
        entity_shaders: HashMap<legion::Entity, Shader>,
        resources: &mut Resources,
    ) -> Vec<Shader> {
        let ts = resources.tape_player.head;
        let mut tape = Tape::default();
        mem::swap(&mut tape, &mut resources.tape_player.tape);
        let shaders = tape.get_shaders(ts, entity_shaders, resources);
        mem::swap(&mut tape, &mut resources.tape_player.tape);
        shaders
    }
}
