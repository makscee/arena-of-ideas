use crate::model::MAX_LIVES;

use super::*;
pub struct Battle {
    config: Config,
    model: Model,
    delta_time: f64,
    // TODO: time or steps limit
}

pub struct BattleResult {
    pub player_won: bool,
    pub units_alive: Vec<Unit>,
}

impl Battle {
    pub fn new(
        config: Config,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: GameRound,
        units_templates: UnitTemplates,
        delta_time: f64,
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
                100000000,
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
            if model.transition || model.current_tick.tick_num > 100 {
                let player_won = model
                    .units
                    .iter()
                    .all(|unit| matches!(unit.faction, Faction::Player));
                return BattleResult {
                    player_won,
                    units_alive: model.units.clone().into_iter().collect(),
                };
            }
        }
    }
}
