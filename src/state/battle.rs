use super::*;

pub struct Battle {
    pub model: Rc<Model>,
    pub view: Rc<View>,
    pub logic: Rc<Logic>,
}

impl State for Battle {
    fn update(&mut self, delta_time: f64) {}
    fn fixed_update(&mut self, delta_time: f64) {}

    fn handle_event(&mut self, event: geng::Event) {}

    fn transition(&mut self) -> Option<geng::Transition> {
        None
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        clear(framebuffer, Some(Rgba::BLUE), None, None);
        self.view.draw(framebuffer);
    }
}
