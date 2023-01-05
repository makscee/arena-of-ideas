use super::*;

pub struct Game {
    pub geng: Geng,
    pub logic: Logic,
    pub model: Model,
    pub assets: Assets,
    pub view: View,
    pub state: StateManager,
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.state.update(delta_time);
    }
    fn fixed_update(&mut self, delta_time: f64) {
        self.state.fixed_update(delta_time);
    }

    fn handle_event(&mut self, event: geng::Event) {
        self.state.handle_event(event);
    }

    fn transition(&mut self) -> Option<geng::Transition> {
        None
    }

    fn ui<'a>(&'a mut self, cx: &'a geng::ui::Controller) -> Box<dyn geng::ui::Widget + 'a> {
        self.state.ui(cx)
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.state.draw(framebuffer);
    }
}
