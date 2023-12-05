use super::*;

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath, Default, Clone)]
#[uuid = "f3c66fac-ac86-4248-a84b-7b76e99a5b63"]
pub struct Ladder {
    pub levels: Vec<String>,
}

impl Ladder {
    pub fn load_current(world: &World) -> (PackedTeam, usize) {
        let save = Save::get(world);
        let ind = save.current_level;
        let initial = Options::get_initial_ladder(world);
        (
            PackedTeam::from_ladder_string(
                if ind < initial.levels.len() {
                    &initial.levels[ind]
                } else {
                    &save.ladder.levels[ind - initial.levels.len()]
                },
                world,
            ),
            ind,
        )
    }

    pub fn levels_left(world: &World) -> usize {
        let save = Save::get(world);
        Self::total_levels(world) - save.current_level
    }

    pub fn total_levels(world: &World) -> usize {
        Options::get_initial_ladder(world).levels.len() + Save::get(world).ladder.levels.len()
    }
}
