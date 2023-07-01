use super::*;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Buff {
    pub name: String,
    pub charges: i32,
    pub rarity: Rarity,
    #[serde(default)]
    pub team_prefix: Option<String>,
}

impl Buff {
    pub fn apply_single(
        &self,
        target: legion::Entity,
        node: &mut Option<Node>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Status::change_charges(target, self.charges, &self.name, node, world, resources)
    }

    pub fn apply_aoe(
        &self,
        faction: Faction,
        node: &mut Option<Node>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        for unit in UnitSystem::collect_faction(world, faction) {
            self.apply_single(unit, node, world, resources);
        }
    }

    pub fn apply_team(
        &self,
        faction: Faction,
        node: &mut Option<Node>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let target = TeamSystem::entity(faction, world).unwrap();
        self.apply_single(target, node, world, resources);
    }

    pub fn apply_team_packed(&self, team: &mut PackedTeam) {
        team.name = format!("{} {}", self.team_prefix.clone().unwrap(), team.name);
        team.statuses.push((self.name.clone(), self.charges));
    }
}
