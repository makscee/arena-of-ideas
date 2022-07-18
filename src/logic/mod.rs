use std::collections::VecDeque;

use super::*;

mod abilities;
mod actions;
mod deaths;
mod effects;
mod particles;
mod spawn;
mod statuses;
mod targeting;
mod time_bombs;
mod util;
mod waves;

pub use effects::*;
pub use util::*;

pub struct Logic<'a> {
    pub model: &'a mut Model,
    pub delta_time: Time,
    pub effects: VecDeque<QueuedEffect<Effect>>,
    pub pressed_keys: Vec<Key>,
    pub render: Option<&'a mut RenderModel>,
}

impl<'a> Logic<'a> {
    pub fn initialize(model: &'a mut Model, config: &Config) {
        let mut logic = Self {
            model,
            delta_time: Time::new(0.0),
            effects: VecDeque::new(),
            pressed_keys: Vec::new(),
            render: None,
        };
        logic.spawn_player(config);
        logic.process();
    }
    pub fn process(&mut self) {
        while self.model.current_tick.tick_time >= Time::new(TICK_TIME) {
            self.tick();
        }

        self.process_particles();
        self.process_statuses();
        self.process_time_bombs();
        self.process_spawns();
        self.process_abilities();
        self.process_targeting();
        self.process_actions();
        self.process_render_positions();
        self.wave_update();
        while self.model.free_revives > 0 {
            if let Some(unit) = self
                .model
                .dead_units
                .iter()
                .filter(|unit| unit.faction == Faction::Player)
                .next()
            {
                self.effects.push_back(QueuedEffect {
                    effect: Effect::Revive(Box::new(ReviveEffect {
                        health: Expr::FindStat {
                            who: Who::Target,
                            stat: UnitStat::MaxHealth,
                        },
                    })),
                    context: EffectContext {
                        caster: None,
                        from: None,
                        target: Some(unit.id),
                        vars: default(),
                        status_id: None,
                    },
                });
                self.model.free_revives -= 1;
            } else {
                break;
            }
        }
        self.process_effects();
        self.process_deaths();
        self.model.time += self.delta_time;
        self.model.current_tick.tick_time += self.delta_time;
    }
    fn tick(&mut self) {
        // TODO: check if some actions did not perform in time
        self.model.current_tick = TickModel::new();
        self.tick_cooldowns();
    }
    fn process_units(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let ids: Vec<Id> = self.model.units.ids().copied().collect();
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }
    fn spawn_player(&mut self, config: &Config) {
        let mut to_spawn = config
            .player
            .iter()
            .map(|unit| (unit, self.model.unit_templates[unit].clone()))
            .collect::<Vec<_>>();

        for unit_type in &config.player {
            self.spawn_unit(unit_type, Faction::Player, Position::zero(Faction::Player));
        }
    }
}

impl Model {
    pub fn update(
        &mut self,
        pressed_keys: Vec<Key>,
        delta_time: Time,
        render: Option<&mut RenderModel>,
    ) {
        let mut logic = Logic {
            model: self,
            delta_time,
            effects: VecDeque::new(),
            pressed_keys,
            render,
        };
        logic.process();
    }
}
