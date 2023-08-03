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

    pub fn apply_single_packed(&self, team: &mut PackedTeam, slot: Option<usize>) {
        let slot = match slot {
            Some(v) => v,
            None => (&mut thread_rng()).gen_range(0..team.units.len()),
        };
        let unit = team.units.get_mut(slot).unwrap();
        unit.statuses.push((self.name.to_owned(), self.charges));
    }

    pub fn apply_aoe_packed(&self, team: &mut PackedTeam) {
        for unit in team.units.iter_mut() {
            unit.statuses.push((self.name.to_owned(), self.charges));
        }
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
