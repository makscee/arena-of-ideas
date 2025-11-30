use super::*;

#[table(name = battle, public)]
pub struct TBattle {
    #[primary_key]
    pub id: u64,
    pub owner: u64,
    pub ts: u64,
    pub hash: u64,
    pub result: Option<bool>,
    pub left_team_name: String,
    pub right_team_name: String,
    pub left_team: String,  // Serialized PackedNodes
    pub right_team: String, // Serialized PackedNodes
}

impl TBattle {
    pub fn create(
        ctx: &mut ServerContext,
        owner: u64,
        left_team: &NTeam,
        right_team: &NTeam,
        hash: u64,
    ) -> Result<u64, String> {
        let left_team = left_team.clone().load_all(ctx)?.take();
        let right_team = right_team.clone().load_all(ctx)?.take();
        let left_team_name = Self::team_name(&left_team);
        let right_team_name = Self::team_name(&right_team);
        let left_packed = left_team.pack().to_string();
        let right_packed = right_team.pack().to_string();

        let battle = TBattle {
            id: ctx.next_id(),
            owner,
            ts: ctx.rctx().timestamp.to_micros_since_unix_epoch() as u64,
            hash,
            result: None,
            left_team_name,
            right_team_name,
            left_team: left_packed,
            right_team: right_packed,
        };

        let battle_id = battle.id;
        ctx.rctx().db.battle().insert(battle);
        Ok(battle_id)
    }

    pub fn update_result(ctx: &ReducerContext, battle_id: u64, result: bool) -> Result<(), String> {
        let Some(mut battle) = ctx.db.battle().id().find(&battle_id) else {
            return Err(format!("Battle {} not found", battle_id));
        };

        battle.result = Some(result);
        ctx.db.battle().id().update(battle);
        Ok(())
    }

    fn team_name(team: &NTeam) -> String {
        let mut houses = vec![];
        if let Ok(team_houses) = team.houses.get() {
            for house in team_houses {
                houses.push(house.house_name.clone());
            }
        }

        if houses.is_empty() {
            format!("Team {}", team.id)
        } else {
            houses.join(", ")
        }
    }
}
