use super::*;

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath, Default, Clone)]
#[uuid = "f3c66fac-ac86-4248-a84b-7b76e99a5b63"]
pub struct Ladder {
    pub teams: Vec<PackedTeam>,
}

impl Ladder {
    pub fn current_level(world: &World) -> (PackedTeam, usize) {
        let save = Save::get(world);
        let ind = save.current_level;
        let initial = Options::get_initial_ladder(world);
        (
            if ind < initial.teams.len() {
                initial.teams[ind].clone()
            } else {
                save.ladder.teams[ind - initial.teams.len()].clone()
            },
            ind,
        )
    }

    pub fn levels_left(world: &World) -> usize {
        let save = Save::get(world);
        Options::get_initial_ladder(world).teams.len() + save.ladder.teams.len()
            - save.current_level
    }
}
