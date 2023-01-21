use geng::prelude::rand::thread_rng;

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
            let mut units = vec![];
            let mut rng = thread_rng();
            let unit = self
                .assets
                .units
                .values()
                .filter(|unit| unit.tier > 0)
                .choose(&mut rng)
                .expect("Units not initialized")
                .clone();
            //Add player units
            units.push(unit.faction(Faction::Player));
            //Add enemy units
            self.assets.rounds[0].enemies.iter().for_each(|enemy| {
                let enemy = self.assets.units.get(enemy).expect("Cant find enemy unit");
                units.push(enemy.clone().faction(Faction::Enemy));
            });
            let mut battle = Battle::new(self.assets.clone(), units);
            while battle.is_battle_over() {
                battle.tick_simulation(logic, view);
            }
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
