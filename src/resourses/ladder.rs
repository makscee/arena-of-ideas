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
                &if ind < initial.levels.len() {
                    initial.levels[ind].clone()
                } else {
                    save.ladder().unwrap().levels[ind - initial.levels.len()].clone()
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
        let ladder_len = Save::get(world)
            .get_ladder_id()
            .ok()
            .and_then(TableLadder::filter_by_id)
            .map(|l| l.levels.len())
            .unwrap_or_default();
        Options::get_initial_ladder(world).levels.len() + ladder_len
    }
}
