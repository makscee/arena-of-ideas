use std::collections::VecDeque;

use super::*;
mod auras;
mod deaths;
mod effects;
mod events;
mod particles;
mod spawn;
mod statuses;
mod tick;
mod time;
mod turn_queue;
mod util;

pub use effects::*;
pub use events::*;
use geng::prelude::itertools::Itertools;
pub use util::*;

enum UnitRef<'a> {
    Id(Id),
    Ref(&'a Unit),
}

pub struct Logic {
    pub model: Model,
    pub delta_time: Time,
    pub effects: VecDeque<QueuedEffect<Effect>>,
    pub paused: bool,
}

impl Logic {
    pub fn initialize(&mut self, events: &mut Events) {
        self.init_time(events);
    }

    pub fn initialize_custom(
        &mut self,
        events: &mut Events,
        custom_player: Vec<UnitTemplate>,
    ) -> Vec<Unit> {
        self.init_time(events);
        self.model.transition = true;
        custom_player
            .iter()
            .map(|template| {
                let unit = Unit::new(
                    &template,
                    self.model.next_id,
                    Position::zero(Faction::Player),
                    &self.model.statuses,
                );
                self.model.next_id += 1;
                unit
            })
            .collect()
    }

    pub fn new(mut model: Model) -> Self {
        Self {
            model,
            delta_time: Time::new(0.0),
            effects: VecDeque::new(),
            paused: false,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.delta_time = Time::new(delta_time as f32);
        self.process_tick();
        self.process_particles();
        self.process_spawns();
        self.process_auras();
        self.process_render_positions();
        self.process_deaths();
        self.process_turn();
        self.process_effects();
        self.process_time();
        self.model.render_model.update(self.delta_time.as_f32())
    }

    fn process_units(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let ids: Vec<Id> = self.model.units.ids().copied().collect();
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }

    fn process_units_random(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let mut ids: Vec<Id> = self.model.units.ids().copied().collect();
        ids.shuffle(&mut global_rng());
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }

    fn process_units_sorted(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let mut units = self.model.units.iter().collect::<Vec<&Unit>>();

        let mut ids: Vec<Id> = units
            .into_iter()
            .sorted_by(|a, b| {
                Ord::cmp(
                    &(a.position.x.abs() - if a.faction == Faction::Player { 1 } else { 0 }),
                    &b.position.x.abs(),
                )
            })
            .map(|unit| unit.id)
            .collect();
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }

    pub fn init_player(&mut self, player: Vec<Unit>) {
        for (index, unit) in player.iter().enumerate() {
            let mut cloned = unit.clone();
            cloned.position.x = index as i64;
            cloned.id = self.model.next_id;
            cloned.shop_unit = Box::new(Some(unit.clone()));
            self.spawn_by_unit(cloned);
        }
    }

    pub fn init_enemies(&mut self, round: GameRound) {
        for unit_type in round.enemies.iter().rev() {
            let unit =
                self.spawn_by_type(&unit_type, Faction::Enemy, Position::zero(Faction::Enemy));
            let unit = self.model.units.get_mut(&unit).unwrap();
            let statuses = round.statuses.iter().map(|status| {
                status.get(&self.model.statuses).clone().attach(
                    Some(unit.id),
                    None,
                    self.model.next_id,
                )
            });
            unit.all_statuses.extend(statuses);
        }
    }

    fn process_render_positions(&mut self) {
        self.process_units(Self::process_unit_render_positions);
    }

    fn process_unit_render_positions(&mut self, unit: &mut Unit) {
        unit.render.render_position +=
            (unit.position.to_world() - unit.render.render_position) * self.delta_time * r32(5.0);
    }
}
