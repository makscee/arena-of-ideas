use std::rc::Rc;

use geng::prelude::itertools::Itertools;

use super::*;

/// Module & State for battle processing
/// Provide units and simulate battle between them,
/// processing Logic effects and producing queue of Visual effects
pub struct Battle {
    pub assets: Rc<Assets>,
    pub units: Collection<Unit>,
    pub strikers: (Option<Id>, Option<Id>),
}

enum StrikePhase {
    Charge,
    Hit,
    Release,
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
    pub fn new(assets: Rc<Assets>, units: Collection<Unit>) -> Self {
        Self {
            assets,
            units,
            strikers: (None, None),
        }
    }

    pub fn is_battle_over(&self) -> bool {
        self.units.iter().unique_by(|unit| unit.faction).count() < 2
    }

    fn get_new_striker(&self, faction: Faction) -> Id {
        self.units
            .iter()
            .filter(|unit| unit.faction == faction)
            .max_by(|x, y| x.slot.partial_cmp(&y.slot).unwrap())
            .unwrap()
            .id
    }

    fn process_strikers(&mut self) {
        if self.strikers.0.is_none() {
            self.strikers.0 = Some(self.get_new_striker(Faction::Player));
        }
        if self.strikers.1.is_none() {
            self.strikers.1 = Some(self.get_new_striker(Faction::Enemy));
        }
    }

    fn get_striker_units(&self) -> (&Unit, &Unit) {
        (
            self.units.get(&self.strikers.0.unwrap()).unwrap(),
            self.units.get(&self.strikers.0.unwrap()).unwrap(),
        )
    }

    fn get_phase_time(phase: StrikePhase) -> Time {
        match phase {
            StrikePhase::Charge => 1.0,
            StrikePhase::Hit => 0.1,
            StrikePhase::Release => 0.7,
        }
    }

    fn add_phase_node(&self, phase: StrikePhase, view: &mut View, model: &VisualNodeModel) {
        let strikers = self.get_striker_units();
        match phase {
            StrikePhase::Charge => {
                let time = Self::get_phase_time(phase);
                view.queue.push_node(
                    model.clone(),
                    vec![
                        Rc::new(AnimateUnitVisualEffect::new(
                            self.strikers.0.unwrap(),
                            vec2(-1.0, 0.0),
                            vec2(-2.0, 0.0),
                            time,
                            AnimateTween::QuartInOut,
                        )),
                        Rc::new(AnimateUnitVisualEffect::new(
                            self.strikers.1.unwrap(),
                            vec2(1.0, 0.0),
                            vec2(2.0, 0.0),
                            time,
                            AnimateTween::QuartInOut,
                        )),
                    ],
                );
            }
            StrikePhase::Hit => {
                let time = Self::get_phase_time(phase);
                view.queue.push_node(
                    model.clone(),
                    vec![
                        Rc::new(AnimateUnitVisualEffect::new(
                            self.strikers.0.unwrap(),
                            vec2(-2.0, 0.0),
                            vec2(-strikers.0.stats.radius, 0.0),
                            time,
                            AnimateTween::Linear,
                        )),
                        Rc::new(AnimateUnitVisualEffect::new(
                            self.strikers.1.unwrap(),
                            vec2(2.0, 0.0),
                            vec2(strikers.1.stats.radius, 0.0),
                            time,
                            AnimateTween::Linear,
                        )),
                    ],
                );
            }
            StrikePhase::Release => {
                let time = Self::get_phase_time(phase);
                view.queue.push_node(
                    model.clone(),
                    vec![
                        Rc::new(AnimateUnitVisualEffect::new(
                            self.strikers.0.unwrap(),
                            vec2(-strikers.0.stats.radius, 0.0),
                            vec2(-1.0, 0.0),
                            time,
                            AnimateTween::QuartOut,
                        )),
                        Rc::new(AnimateUnitVisualEffect::new(
                            self.strikers.1.unwrap(),
                            vec2(strikers.1.stats.radius, 0.0),
                            vec2(1.0, 0.0),
                            time,
                            AnimateTween::QuartOut,
                        )),
                    ],
                );
            }
        };
    }

    /// continue simulation and produce 1 new VisualNode
    pub fn tick_simulation(&mut self, logic: &mut Logic, view: &mut View) {
        let model = VisualNodeModel {
            units: self
                .units
                .iter()
                .map(|u| UnitRender::new_from_unit(u))
                .collect(),
        };

        self.process_strikers();
        self.add_phase_node(StrikePhase::Charge, view, &model);
        self.add_phase_node(StrikePhase::Hit, view, &model);
        self.add_phase_node(StrikePhase::Release, view, &model);
    }
}
