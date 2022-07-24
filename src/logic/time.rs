use super::*;

impl Logic {
    pub fn init_time(&mut self, events: &mut Events) {
        events.add_listener(
            GameEvent::Pause,
            Box::new(|logic| logic.paused = !logic.paused),
        );
    }
    pub fn process_time(&mut self) {
        self.model.time += self.delta_time;
        self.model.current_tick.tick_time += self.delta_time;
    }
}
