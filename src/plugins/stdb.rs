use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StdbData>()
            .add_systems(Update, Self::update);
    }
}

#[derive(Resource, Default)]
struct StdbData {
    nodes_queue: Vec<TNode>,
}

impl StdbPlugin {
    fn update(world: &mut World) {
        world.resource_scope(|world, mut d: Mut<StdbData>| {
            d.nodes_queue.retain(|node| {
                let mut cur_node = node.clone();
                let mut id = node.id;
                loop {
                    let parent = cur_node.parent.get_node().unwrap();
                    if parent.kind.to_kind().is_component(cur_node.kind.to_kind()) {
                        cur_node = parent;
                        id = cur_node.id;
                    } else {
                        break;
                    }
                }
                let Some(parent) = world.get_id_link(node.parent) else {
                    return true;
                };
                if let Some(entity) = world.get_id_link(id) {
                    node.unpack(entity, world);
                    return false;
                }
                node.unpack(world.spawn_empty().set_parent(parent).id(), world);
                false
            });
        });
    }
}

pub fn subscribe_game(on_success: impl FnOnce() + Send + Sync + 'static) {
    info!("Apply stdb subscriptions");
    cn().subscription_builder()
        .on_error(|_, error| error.to_string().notify_error_op())
        .on_applied(move |e| {
            info!("Subscription applied");
            on_success();
            subscribe_table_updates();
            OperationsPlugin::add(|world| {
                dbg!(All::load_recursive(0).unwrap()).unpack(world.spawn_empty().id(), world);
                let pid = player_id();
                let entity = world
                    .get_id_link(pid)
                    .expect(&format!("Player#{pid} not found"));
                save_player_entity(entity);
            });
        })
        .subscribe_to_all_tables();
}
fn subscribe_table_updates() {
    let db = cn().db();
    db.nodes_world().on_insert(|_, node| {
        info!("Node inserted {}#{}", node.kind, node.id);
        let node = node.clone();
        OperationsPlugin::add(move |world| {
            world.resource_mut::<StdbData>().nodes_queue.push(node);
        });
    });
    db.nodes_world().on_update(|_, _, node| {
        info!("Node updated {}#{}", node.kind, node.id);
        let node = node.clone();
        OperationsPlugin::add(move |world| {
            let Some(entity) = world.get_id_link(node.id) else {
                error!("Failed to update entity: id link not found");
                return;
            };
            node.unpack(entity, world);
        });
    });
    db.nodes_world().on_delete(|_, node| {
        info!("Node deleted {}#{}", node.kind, node.id);
        let node = node.clone();
        OperationsPlugin::add(move |world| {
            let Some(entity) = world.get_id_link(node.id) else {
                error!("Failed to delete entity: id link not found");
                return;
            };
            if let Ok(e) = world.get_entity_mut(entity) {
                e.try_despawn_recursive();
            }
        });
    });
    db.battle().on_insert(|_, row| {
        todo!();
    });
}

pub fn subscribe_reducers() {
    cn().reducers.on_match_insert(|e| {
        e.event.notify_error();
    });
    cn().reducers.on_match_edit_fusions(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_reorder(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_incubator_vote(|e, _, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_incubator_push(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
}
