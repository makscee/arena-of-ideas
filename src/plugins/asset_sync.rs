use crate::module_bindings::{
    sync_abilities, sync_houses, sync_statuses, sync_units, sync_vfxs, TableUnit,
};

use super::*;

pub struct AssetsSyncPlugin;

impl Plugin for AssetsSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UnitSync), do_sync);
    }
}

fn do_sync(world: &mut World) {
    debug!("Assets Sync start");
    let mut data: Vec<TableUnit> = default();
    for (_, asset) in Pools::get(world).heroes.iter() {
        data.push(asset.clone().into());
    }
    debug!("Sync {} units", data.len());
    sync_units(data);

    let mut data: Vec<module_bindings::House> = default();
    for (name, asset) in Pools::get(world).houses.iter() {
        data.push(module_bindings::House {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }
    debug!("Sync {} houses", data.len());
    sync_houses(data);

    let mut data: Vec<module_bindings::Ability> = default();
    for (name, asset) in Pools::get(world).abilities.iter() {
        data.push(module_bindings::Ability {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }
    debug!("Sync {} abilities", data.len());
    sync_abilities(data);

    let mut data: Vec<module_bindings::Statuses> = default();
    for (name, asset) in Pools::get(world).statuses.iter() {
        data.push(module_bindings::Statuses {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }
    debug!("Sync {} statuses", data.len());
    sync_statuses(data);

    let mut data: Vec<module_bindings::Vfx> = default();
    for (name, asset) in Pools::get(world).vfx.iter() {
        data.push(module_bindings::Vfx {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }
    debug!("Sync {} vfxs", data.len());
    sync_vfxs(data);

    world.send_event(AppExit);
}
