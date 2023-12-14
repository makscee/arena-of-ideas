use super::*;

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath, Default, Clone)]
#[uuid = "f3c66fac-ac86-4248-a84b-7b76e99a5b63"]
pub struct Ladder {
    pub levels: Vec<String>,
}

impl Ladder {
    pub fn load_current(world: &World) -> (PackedTeam, usize) {
        let save = Save::get(world).unwrap();
        let ind = save.climb.defeated;
        (
            PackedTeam::from_ladder_string(&save.climb.levels[ind], world),
            ind,
        )
    }

    pub fn levels_left(world: &World) -> usize {
        let save = Save::get(world).unwrap();
        Self::total_levels(world) - save.climb.defeated
    }

    pub fn total_levels(world: &World) -> usize {
        Save::get(world).unwrap().climb.levels.len()
    }
}
