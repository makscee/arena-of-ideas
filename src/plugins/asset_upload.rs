use crate::module_bindings::{once_on_sync_data, sync_data, TableUnit};

use super::*;

pub struct AssetsUploadPlugin;

impl Plugin for AssetsUploadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::AssetSync), do_sync);
    }
}

fn do_sync(world: &mut World) {
    debug!("Assets Sync start");
    let mut units: Vec<TableUnit> = default();
    for (_, asset) in Pools::get(world).heroes.iter() {
        units.push(asset.clone().into());
    }
    let mut houses: Vec<module_bindings::House> = default();
    for (name, asset) in Pools::get(world).houses.iter() {
        houses.push(module_bindings::House {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }

    let mut abilities: Vec<module_bindings::Ability> = default();
    for (name, asset) in Pools::get(world).abilities.iter() {
        abilities.push(module_bindings::Ability {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }

    let mut summons: Vec<module_bindings::Summon> = default();
    for (name, asset) in Pools::get(world).summons.iter() {
        summons.push(module_bindings::Summon {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }

    let mut statuses: Vec<module_bindings::Statuses> = default();
    for (name, asset) in Pools::get(world).statuses.iter() {
        statuses.push(module_bindings::Statuses {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }

    let mut vfxs: Vec<module_bindings::Vfx> = default();
    for (name, asset) in Pools::get(world).vfx.iter() {
        vfxs.push(module_bindings::Vfx {
            name: name.clone(),
            data: ron::to_string(asset).unwrap(),
        });
    }
    sync_data(houses, abilities, statuses, summons, units, vfxs);
    once_on_sync_data(|_, _, status, _, _, _, _, _, _| {
        match status {
            spacetimedb_sdk::reducer::Status::Committed => {
                debug!("Sync Success");
            }
            spacetimedb_sdk::reducer::Status::Failed(e) => {
                error!("Sync Failure: {e}");
            }
            spacetimedb_sdk::reducer::Status::OutOfEnergy => panic!(),
        };
        OperationsPlugin::add(|w| {
            w.send_event(AppExit);
        });
    });
}
