use super::*;

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath, Default, Clone)]
#[uuid = "f3c66fac-ac86-4248-a84b-7b76e99a5b63"]
pub struct Tower {
    pub levels: Vec<String>,
}

impl Tower {
    pub fn load_current(world: &World) -> (PackedTeam, usize) {
        let save = Save::get(world).unwrap();

        let ind = save.climb.defeated;

        (
            if ind == save.climb.levels.len() {
                save.climb.owner_team.unwrap()
            } else {
                PackedTeam::from_tower_string(&save.climb.levels[ind], world)
            },
            ind,
        )
    }

    pub fn levels_left(world: &World) -> usize {
        let save = Save::get(world).unwrap();
        Self::total_levels(world) - save.climb.defeated
    }

    pub fn total_levels(world: &World) -> usize {
        let save = Save::get(world).unwrap();
        save.climb.levels.len()
            + if matches!(save.mode, GameMode::RandomTower { .. }) {
                1
            } else {
                0
            }
    }
}
