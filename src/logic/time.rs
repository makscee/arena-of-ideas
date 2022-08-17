use super::*;
const SPEED_1: f32 = 1.0;
const SPEED_2: f32 = 2.0;
const SPEED_3: f32 = 4.0;

impl Logic {
    pub fn init_time(&mut self, events: &mut Events) {
        events.add_listener(
            GameEvent::Pause,
            Box::new(|logic| logic.paused = !logic.paused),
        );
        events.add_listener(
            GameEvent::Speed1,
            Box::new(|logic| logic.model.time_modifier = SPEED_1),
        );
        events.add_listener(
            GameEvent::Speed2,
            Box::new(|logic| logic.model.time_modifier = SPEED_2),
        );
        events.add_listener(
            GameEvent::Speed3,
            Box::new(|logic| logic.model.time_modifier = SPEED_3),
        );
    }
    pub fn process_time(&mut self) {
        self.model.time += self.delta_time;
        self.model.current_tick.tick_time += self.delta_time;
    }
}
