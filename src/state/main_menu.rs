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
            let battle = Battle {
                assets: self.assets.clone(),
            };
            let mut battle_units = Collection::new();
            battle_units.insert(Unit::new(
                0,
                "Test".to_owned(),
                UnitStats {},
                Faction::Player,
            ));
            Battle::start_simulation(battle_units, logic);
            Battle::tick_simulation(logic, view);
            Battle::tick_simulation(logic, view);
            Battle::tick_simulation(logic, view);
            Battle::tick_simulation(logic, view);
            Battle::tick_simulation(logic, view);
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
