use spacetimedb_sdk::table::TableWithPrimaryKey;

use super::*;

pub struct TeamSyncPlugin;

impl Plugin for TeamSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TeamSyncResource>()
            .add_systems(OnEnter(GameState::Title), Self::setup);
    }
}

#[derive(Resource, Default)]
struct TeamSyncResource {
    synced: HashMap<u64, Faction>,
}

impl TeamSyncPlugin {
    pub fn subscribe(team_id: u64, faction: Faction, world: &mut World) {
        let synced = &world.resource::<TeamSyncResource>().synced;
        if synced.contains_key(&team_id) {
            error!("Team#{team_id} is already subscribed");
            return;
        }
        if let Some(id) = synced
            .iter()
            .find_map(|(id, f)| if faction.eq(f) { Some(*id) } else { None })
        {
            debug!("Faction {faction} already subscribed, replacing");
            Self::unsubscribe(id, world);
        }
        world
            .resource_mut::<TeamSyncResource>()
            .synced
            .insert(team_id, faction);
        Self::sync_team(team_id, world);
        debug!("Subscribe team#{team_id}");
    }
    pub fn unsubscribe(team_id: u64, world: &mut World) {
        debug!("Unsubscribe team#{team_id}");
        if let Some(faction) = world
            .resource_mut::<TeamSyncResource>()
            .synced
            .remove(&team_id)
        {
            TeamPlugin::despawn(faction, world);
        }
    }
    pub fn unsubscribe_all(world: &mut World) {
        debug!("TeamSync: unsubscribe all");
        for id in world
            .resource_mut::<TeamSyncResource>()
            .synced
            .keys()
            .copied()
            .collect_vec()
        {
            Self::unsubscribe(id, world);
        }
    }
    fn setup() {
        TTeam::on_update(|_, new, _| {
            let id = new.id;
            OperationsPlugin::add(move |world| {
                Self::sync_team(id, world);
            });
        });
    }
    fn sync_team(team_id: u64, world: &mut World) {
        let Some(faction) = world
            .resource_mut::<TeamSyncResource>()
            .synced
            .get(&team_id)
            .copied()
        else {
            return;
        };
        let Some(team) = TTeam::find_by_id(team_id) else {
            error!("Team#{team_id} not found");
            return;
        };
        Self::sync_units(team.units, faction, world);
    }
    fn sync_units(units: Vec<FusedUnit>, faction: Faction, world: &mut World) {
        let team_units: IndexMap<u64, FusedUnit> =
            IndexMap::from_iter(units.into_iter().map(|u| (u.id, u)));
        let mut world_units: HashMap<u64, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(faction, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        world_units.retain(|id, e| {
            if !team_units.contains_key(id) {
                UnitPlugin::despawn(*e, world);
                return false;
            } else {
                return true;
            }
        });
        for (slot, (id, unit)) in team_units.into_iter().enumerate() {
            let slot = slot as i32;
            if let Some(entity) = world_units.get(&id) {
                Self::sync_unit(slot, unit, *entity, world);
            } else {
                PackedUnit::from(unit).unpack(
                    TeamPlugin::entity(faction, world),
                    Some(slot),
                    Some(id),
                    world,
                );
            }
        }
    }
    fn sync_unit(slot: i32, unit: FusedUnit, entity: Entity, world: &mut World) {
        let mut state = VarState::get_mut(entity, world);
        state.set_int(VarName::Slot, slot.into());
        state.set_int(VarName::Hp, unit.hp);
        state.set_int(VarName::Pwr, unit.pwr);
        let new_xp = unit.xp as i32;
        let old_xp = state
            .get_value_last(VarName::Xp)
            .unwrap()
            .get_int()
            .unwrap();
        let new_lvl = unit.lvl as i32;
        let old_lvl = state
            .get_value_last(VarName::Lvl)
            .unwrap()
            .get_int()
            .unwrap();
        let level_changed = old_lvl != new_lvl;
        let xp_changed = old_xp != new_xp;

        if level_changed {
            state.set_int(VarName::Lvl, new_lvl);
        }
        if xp_changed {
            state.set_int(VarName::Xp, new_xp);
        }
        if level_changed {
            TextColumnPlugin::add(
                entity,
                format!("+{} Lvl", new_lvl - old_lvl).cstr_cs(PURPLE, CstrStyle::Bold),
                world,
            );
        } else if xp_changed {
            TextColumnPlugin::add(
                entity,
                format!("+{} Xp", new_xp - old_xp).cstr_cs(LIGHT_PURPLE, CstrStyle::Bold),
                world,
            );
        }
    }
}
