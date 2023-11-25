use super::*;

#[derive(Serialize, Deserialize, Debug, TypeUuid, TypePath, Default)]
#[uuid = "f3c66fac-ac86-4248-a84b-7b76e99a5b63"]
pub struct Ladder {
    pub teams: Vec<PackedTeam>,
}

impl Ladder {
    pub fn current_level(world: &World) -> PackedTeam {
        let save = Save::get(world).unwrap();
        let ind = save.current_level;
        let initial = Options::get_initial_ladder(world);
        if ind < initial.teams.len() {
            initial.teams[ind].clone()
        } else {
            save.ladder.teams[ind - initial.teams.len()].clone()
        }
    }

    pub fn is_on_last_level(world: &World) -> bool {
        let save = Save::get(world).unwrap();
        save.current_level + 1
            == Options::get_initial_ladder(world).teams.len() + save.ladder.teams.len()
    }
}
