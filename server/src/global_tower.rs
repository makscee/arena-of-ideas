use super::*;

#[spacetimedb(table)]
pub struct GlobalTower {
    #[primarykey]
    number: u64,
    owner: Option<u64>,
    floor: TowerFloor,
}

#[derive(SpacetimeType, PartialEq, Eq)]
pub enum TowerFloor {
    Enemy(String),
    Team(String),
}

#[spacetimedb(reducer)]
fn extend_global_tower(ctx: ReducerContext, enemy: String, team: String) -> Result<(), String> {
    let user = User::find_by_identity(&ctx.sender)?;
    let number = GlobalTower::iter().count() as u64;
    GlobalTower::insert(GlobalTower {
        number: number + 1,
        owner: None,
        floor: TowerFloor::Enemy(enemy),
    })?;
    GlobalTower::insert(GlobalTower {
        number: number + 2,
        owner: Some(user.id),
        floor: TowerFloor::Team(team),
    })?;
    Ok(())
}

impl GlobalTower {
    pub fn init() -> Result<(), String> {
        GlobalTower::insert(GlobalTower {
            number: 1,
            owner: None,
            floor: TowerFloor::Enemy("Bug_1".to_owned()),
        })?;
        GlobalTower::insert(GlobalTower {
            number: 2,
            owner: None,
            floor: TowerFloor::Enemy("Bug_5".to_owned()),
        })?;
        GlobalTower::insert(GlobalTower {
            number: 3,
            owner: None,
            floor: TowerFloor::Enemy("Snake_2".to_owned()),
        })?;
        Ok(())
    }
}
