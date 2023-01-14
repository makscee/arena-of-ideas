use super::*;

pub struct Battle {
    pub assets: Rc<Assets>,
}

impl State for Battle {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, view: &View, logic: &Logic) {
        clear(framebuffer, Some(Rgba::GREEN), None, None);
        view.draw(framebuffer, logic.model.game_time);
    }

    fn update(&mut self, delta_time: Time, logic: &mut Logic, view: &mut View) {
        view.update(logic.model.game_time);
    }
}
