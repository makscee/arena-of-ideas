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

    fn transition(&mut self, logic: &mut Logic, view: &mut View) -> Option<Transition> {
        if self.transition {
            let mut units = Collection::new();
            units.insert(Unit::new_test(0, Faction::Player));
            units.insert(Unit::new_test(1, Faction::Enemy));
            let mut battle = Battle::new(self.assets.clone(), units);
            battle.tick_simulation(logic, view);
            battle.tick_simulation(logic, view);
            battle.tick_simulation(logic, view);
            battle.tick_simulation(logic, view);
            battle.tick_simulation(logic, view);
            battle.tick_simulation(logic, view);
            Some(Transition::Switch(Box::new(battle)))
        } else {
            None
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer, view: &View, logic: &Logic) {
        clear(framebuffer, Some(Rgba::MAGENTA), None, None);
        // self.view.draw(framebuffer);
    }
}
