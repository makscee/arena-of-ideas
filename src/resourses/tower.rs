use crate::module_bindings::GlobalTower;

use super::*;

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath, Default, Clone)]
#[uuid = "f3c66fac-ac86-4248-a84b-7b76e99a5b63"]
pub struct Tower {
    pub levels: Vec<String>,
}

impl Tower {
    pub fn load_current(world: &World) -> (PackedTeam, usize) {
        let save = Save::get(world).unwrap();

        let ind = save.climb.defeated + 1;
        let floor = GlobalTower::filter_by_number(ind as u64).unwrap().floor;
        (
            match floor {
                module_bindings::TowerFloor::Enemy(team) => {
                    PackedTeam::from_tower_string(&team, world)
                }
                module_bindings::TowerFloor::Team(team) => ron::from_str(&team).unwrap(),
            },
            ind,
        )
    }

    pub fn levels_left(world: &World) -> usize {
        let save = Save::get(world).unwrap();
        Self::total_levels() - save.climb.defeated
    }

    pub fn total_levels() -> usize {
        GlobalTower::count()
    }
}
