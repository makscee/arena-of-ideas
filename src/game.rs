use super::*;

pub struct Game {
    pub geng: Geng,
    pub logic: Logic,
    pub model: Model,
    pub assets: Assets,
    pub view: View,
    pub state: StateManager,
}

impl State for Game {
    fn update(&mut self, delta_time: f64) {}
    fn fixed_update(&mut self, delta_time: f64) {}

    fn handle_event(&mut self, event: geng::Event) {}

    fn transition(&mut self) -> Option<geng::Transition> {
        None
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        clear(framebuffer, Some(Rgba::WHITE), None, None);
        
    }
}
