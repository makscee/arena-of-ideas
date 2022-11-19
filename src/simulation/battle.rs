use crate::{
    assets::Sounds,
    model::{Position, MAX_LIVES},
    shop::Shop,
};

use super::*;
pub struct Battle {
    config: Config,
    model: Model,
    delta_time: f32,
    round: GameRound,
    player: Vec<Unit>,
}

#[derive(Serialize, Clone)]
pub struct BattleResult {
    pub player: Vec<Unit>,
    pub enemy: Vec<UnitType>,
    pub player_won: bool,
    pub damage_sum: i32,
    pub health_sum: i32,
    pub units_alive: Vec<UnitType>,
    pub stats_before: HashMap<UnitType, String>,
    pub stats_after: HashMap<UnitType, String>,
    pub round: String,
}

impl Battle {
    pub fn new(
        config: Config,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: GameRound,
        units_templates: UnitTemplates,
        delta_time: f32,
        lives: i32,
        player: Vec<Unit>,
    ) -> Self {
        let rounds = vec![round.clone()];
        let shop = Shop::new(1, &units_templates);
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
                shop,
            ),
            delta_time,
            round,
            player,
        }
    }

    pub fn run(mut self) -> BattleResult {
        let mut logic = Logic::new(self.model.clone(), Sounds { map: hashmap!() });
        let mut events = Events::new(vec![]);
        let mut stats_before: HashMap<UnitType, String> = hashmap! {};
        let mut stats_after: HashMap<UnitType, String> = hashmap! {};

        self.player.iter_mut().for_each(|unit| {
            stats_before.insert(
                unit.unit_type.clone(),
                format!(
                    "{}/{},L{},S{},P{},ID{}",
                    unit.stats.attack,
                    unit.stats.health,
                    unit.stats.level(),
                    unit.stats.stacks,
                    unit.position.x,
                    unit.id,
                ),
            );

            unit.id = self.model.next_id;
            self.model.next_id += 1;
        });

        logic.initialize(&mut events);
        logic.model.team = self.player.clone();
        logic.init_round(self.round.clone());

        loop {
            logic.update(self.delta_time);
            let model = &logic.model;

            if model.lives <= 0 || model.transition {
                let player_won = model
                    .units
                    .iter()
                    .all(|unit| matches!(unit.faction, Faction::Player));

                let units_alive: Vec<String> = model
                    .units
                    .clone()
                    .into_iter()
                    .map(|unit| {
                        stats_after.insert(
                            unit.unit_type.clone(),
                            format!(
                                "{}/{},L{},S{},P{},ID{}",
                                unit.stats.attack,
                                unit.stats.health,
                                unit.stats.level(),
                                unit.stats.stacks,
                                unit.position.x,
                                unit.id,
                            ),
                        );
                        unit.unit_type
                    })
                    .collect();
                //todo: revert to team units
                let player = model.team.clone();
                let units_count = if player_won {
                    units_alive.len() as i32
                } else {
                    -(units_alive.len() as i32)
                };
                return BattleResult {
                    player,
                    enemy: self.round.enemies.clone(),
                    damage_sum: units_count.clone(),
                    health_sum: units_count.clone(),
                    player_won,
                    round: self.round.name,
                    units_alive,
                    stats_before,
                    stats_after,
                };
            }
        }
    }
}
