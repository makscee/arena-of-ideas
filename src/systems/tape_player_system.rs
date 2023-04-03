use super::*;

pub struct TapePlayerSystem;

impl TapePlayerSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for TapePlayerSystem {
    fn update(&mut self, _: &mut legion::World, resources: &mut Resources) {
        let head = &mut resources.tape_player.head;
        if resources.input.down_keys.contains(&Space) {
            resources.tape_player.mode = match resources.tape_player.mode {
                TapePlayMode::Play => TapePlayMode::Stop { ts: *head },
                TapePlayMode::Stop { .. } => TapePlayMode::Play,
            };
        } else if resources.input.pressed_keys.contains(&Left)
            || resources.input.pressed_keys.contains(&Right)
        {
            let old_ts = match resources.tape_player.mode {
                TapePlayMode::Play => *head,
                TapePlayMode::Stop { ts } => ts,
            };
            let direction = match resources.input.pressed_keys.contains(&Left) {
                true => -1.0,
                false => 1.0,
            };
            resources.tape_player.mode = TapePlayMode::Stop {
                ts: old_ts + direction * resources.delta_time * resources.options.rewind_add_speed,
            };
        }
        match resources.tape_player.mode {
            TapePlayMode::Play => *head += resources.delta_time,
            TapePlayMode::Stop { ts } => {
                *head += (ts - *head) * resources.delta_time * resources.options.rewind_speed
            }
        }
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
