use super::*;

pub struct MainMenu {
    pub assets: Rc<Assets>,
    pub transition: bool,
}

impl State for MainMenu {
    fn handle_event(&mut self, event: Event, logic: &mut Logic) {
        match event {
            Event::KeyDown { key } => {
                self.transition = true;
            }
            _ => {}
        }
    }

    fn transition(&mut self, logic: &mut Logic) -> Option<Transition> {
        if self.transition {
            Some(Transition::Switch(Box::new(Battle {
                assets: self.assets.clone(),
            })))
        } else {
            None
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, view: &View, logic: &Logic) {
        clear(framebuffer, Some(Rgba::MAGENTA), None, None);
        // self.view.draw(framebuffer);
    }
}
