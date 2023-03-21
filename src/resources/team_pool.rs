use super::*;

pub struct TeamPool {
    teams: HashMap<Faction, Team>,
}

impl TeamPool {
    pub fn new(teams: HashMap<Faction, Team>) -> Self {
        Self { teams }
    }

    pub fn get_ability_var_int(
        house: &HouseName,
        ability: &str,
        var: &VarName,
        faction: &Faction,
        resources: &Resources,
    ) -> i32 {
        resources
            .team_pool
            .teams
            .get(faction)
            .and_then(|x| x.try_get_ability_var_int(house, ability, var))
            .unwrap_or(
                resources
                    .house_pool
                    .try_get_ability_var_int(house, ability, var)
                    .expect(&format!(
                        "Failed to get ability var {:?} {:?} {} {}",
                        faction, house, ability, var
                    )),
            )
    }

    pub fn set_ability_var_int(
        house: &HouseName,
        ability: &str,
        var: &VarName,
        value: i32,
        faction: &Faction,
        resources: &mut Resources,
    ) {
        resources
            .team_pool
            .teams
            .get_mut(faction)
            .unwrap()
            .ability_state
            .set_var(house, ability, *var, Var::Int(value));
    }

    pub fn refresh_team(faction: &Faction, world: &legion::World, resources: &mut Resources) {
        let mut team = resources
            .team_pool
            .teams
            .remove(faction)
            .unwrap_or_default();
        team.units = Team::pack_entries(faction, world, resources);
        resources.team_pool.teams.insert(*faction, team);
    }

    pub fn save_team(faction: Faction, team: Team, resources: &mut Resources) {
        resources.team_pool.teams.insert(faction, team);
    }

    pub fn load_team(faction: &Faction, world: &mut legion::World, resources: &mut Resources) {
        resources
            .team_pool
            .teams
            .get(faction)
            .unwrap()
            .clone()
            .unpack_entries(faction, world, resources);
    }

    pub fn get_team(faction: Faction, resources: &Resources) -> &Team {
        Self::try_get_team(faction, resources).unwrap()
    }

    pub fn try_get_team(faction: Faction, resources: &Resources) -> Option<&Team> {
        resources.team_pool.teams.get(&faction)
    }
}
