use super::*;

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as Time;
        self.logic.model.game_time += delta_time;
        self.state
            .update(delta_time, &mut self.logic, &mut self.view);
    }

    fn handle_event(&mut self, event: Event) {
        self.state.handle_event(event, &mut self.logic);
    }

    fn transition(&mut self) -> Option<geng::Transition> {
        self.state.transition(&mut self.logic);
        None
    }

    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        self.state.ui(cx)
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.state
            .draw(framebuffer, &mut self.view, &mut self.logic);
    }
}
