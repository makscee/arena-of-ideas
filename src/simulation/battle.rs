use crate::model::{Position, MAX_LIVES};

use super::*;
pub struct Battle {
    config: Config,
    model: Model,
    delta_time: f64,
    round: GameRound,
}

#[derive(Serialize, Clone)]
pub struct BattleResult {
    pub player: Vec<UnitType>,
    pub player_won: bool,
    pub damage_sum: i32,
    pub health_sum: i32,
    pub units_alive: Vec<UnitType>,
    pub round: String,
}

impl Battle {
    pub fn new(
        config: Config,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: GameRound,
        units_templates: UnitTemplates,
        delta_time: f64,
        lives: i32,
    ) -> Self {
        let rounds = vec![round.clone()];
        Self {
            config: config.clone(),
            model: Model::new(
                config,
                units_templates,
                clan_effects,
                statuses,
                0,
                rounds,
                RenderModel::new(),
                1.0,
                lives,
            ),
            delta_time,
            round,
        }
    }

    pub fn run(mut self) -> BattleResult {
        let mut logic = Logic::new(self.model.clone());
        let mut events = Events::new(vec![]);
        logic.initialize(&mut events);
        self.config.player.iter().for_each(|unit_config| {
            logic.spawn_by_type(unit_config, Position::zero(Faction::Player));
        });
        self.round.enemies.iter().rev().for_each(|unit_config| {
            logic.spawn_by_type(unit_config, Position::zero(Faction::Enemy));
        });

        loop {
            logic.update(self.delta_time);
            let model = &logic.model;
            if model.lives <= 0 || model.transition || model.current_tick.tick_num > 100 {
                let player_won = model
                    .units
                    .iter()
                    .all(|unit| matches!(unit.faction, Faction::Player));

                let units_alive: Vec<String> = model
                    .units
                    .clone()
                    .into_iter()
                    .map(|unit| unit.unit_type)
                    .collect();
                let units_count = if player_won {
                    units_alive.len() as i32
                } else {
                    -(units_alive.len() as i32)
                };
                return BattleResult {
                    player: self.model.config.player.clone(),
                    damage_sum: units_count.clone(),
                    health_sum: units_count.clone(),
                    player_won,
                    round: self.round.name,
                    units_alive,
                };
            }
        }
    }
}
