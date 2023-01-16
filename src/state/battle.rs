use std::rc::Rc;

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

impl Battle {
    pub fn new(assets: Rc<Assets>) -> Self {
        Self { assets }
    }

    // add units to model and initialize simulation
    pub fn start_simulation(battle_units: Collection<Unit>, logic: &mut Logic) {
        logic.model.battle_units = battle_units;
    }

    pub fn tick_simulation(logic: &mut Logic, view: &mut View) {
        let model = VisualNodeModel {
            units: logic
                .model
                .battle_units
                .iter()
                .map(|u| UnitRender::new_from_unit(u))
                .collect(),
        };
        let effects = AnimateUnitVisualEffect::new(0, vec2(0.0, 0.0), vec2(-1.0, 0.0));
        view.queue.push_node(model.clone(), vec![Rc::new(effects)]);
        let effects = AnimateUnitVisualEffect::new(0, vec2(-1.0, 0.0), vec2(0.0, 0.0));
        view.queue.push_node(model.clone(), vec![Rc::new(effects)]);
    }
}
