use crate::model::MAX_LIVES;

use super::*;
pub struct Battle {
    config: Config,
    model: Model,
    delta_time: f64,
    // TODO: time or steps limit
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
        Self {
            config: config.clone(),
            model: Model::new(
                config,
                units_templates,
                clan_effects,
                statuses,
                round,
                RenderModel::new(),
                1.0,
                lives,
            ),
            delta_time,
        }
    }

    pub fn run(mut self) -> BattleResult {
        let mut logic = Logic::new(self.model.clone());
        let mut events = Events::new(vec![]);
        logic.initialize(
            &mut events,
            None,
            self.config.player.clone(),
            self.model.round.clone(),
        );

        loop {
            logic.update(self.delta_time);
            let model = &logic.model;
            if model.lives <= 0 || model.transition || model.current_tick.tick_num > 100 {
                let player_won = model
                    .units
                    .iter()
                    .all(|unit| matches!(unit.faction, Faction::Player));
                let units_alive = model
                    .units
                    .clone()
                    .into_iter()
                    .map(|unit| unit.unit_type)
                    .collect();

                let mut health_sum = model
                    .units
                    .clone()
                    .into_iter()
                    .map(|unit| unit.stats.health)
                    .sum::<i32>();

                let mut damage_sum = model
                    .units
                    .clone()
                    .into_iter()
                    .map(|unit| unit.stats.attack)
                    .sum::<i32>();
                if !player_won {
                    health_sum = -health_sum;
                    damage_sum = -damage_sum;
                }
                return BattleResult {
                    player: self.model.config.player.clone(),
                    damage_sum: damage_sum as i32,
                    health_sum: health_sum as i32,
                    player_won,
                    round: self.model.round.name,
                    units_alive,
                };
            }
        }
    }
}
