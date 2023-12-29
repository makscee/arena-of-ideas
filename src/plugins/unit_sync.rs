use crate::module_bindings::{give_right, sync_units};

use super::*;

pub struct UnitSyncPlugin;

impl Plugin for UnitSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UnitSync), do_sync);
    }
}

fn do_sync(world: &mut World) {
    debug!("Sync start");
    LoginPlugin::connect();
    // give_right(identity().unwrap(), module_bindings::UserRight::UnitSync);
    let mut units: Vec<module_bindings::Unit> = default();
    for (name, unit) in Pools::get(world).heroes.iter() {
        units.push(module_bindings::Unit {
            name: name.clone(),
            data: ron::to_string(unit).unwrap(),
            pool: module_bindings::UnitPool::Hero,
        });
    }
    for (name, unit) in Pools::get(world).enemies.iter() {
        units.push(module_bindings::Unit {
            name: name.clone(),
            data: ron::to_string(unit).unwrap(),
            pool: module_bindings::UnitPool::Enemy,
        });
    }
    sync_units(units);
    world.send_event(AppExit);
}
