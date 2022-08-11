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
    pub lives: i32,
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
                return BattleResult {
                    player: self.model.config.player.clone(),
                    lives: model.lives,
                    player_won,
                    round: self.model.round.name,
                    units_alive: model
                        .units
                        .clone()
                        .into_iter()
                        .map(|unit| unit.unit_type)
                        .collect(),
                };
            }
        }
    }
}
