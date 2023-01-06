use std::default;

use super::*;

pub struct MainMenu {
    pub model: Rc<Model>,
    pub view: Rc<View>,
    pub logic: Rc<Logic>,
    pub transition: bool,
}

impl State for MainMenu {
    fn update(&mut self, delta_time: f64) {}
    fn fixed_update(&mut self, delta_time: f64) {}

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            Event::KeyDown { key } => {
                self.transition = true;
            }
            _ => {}
        }
    }

    fn transition(&mut self) -> Option<geng::Transition> {
        if self.transition {
            Some(Transition::Switch(Box::new(Battle {
                model: self.model.clone(),
                view: self.view.clone(),
                logic: self.logic.clone(),
            })))
        } else {
            None
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        clear(framebuffer, Some(Rgba::MAGENTA), None, None);
        self.view.draw(framebuffer);
    }
}
