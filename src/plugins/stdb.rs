use bevy::ecs::event::{Event, Events};
use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StdbData>()
            .init_resource::<Events<StdbEvent>>()
            .add_systems(Update, Self::update);
    }
}

#[derive(Resource, Default)]
struct StdbData {
    nodes_queue: Vec<TNode>,
    on_empty: Vec<Operation>,
}

pub enum StdbChange {
    Update,
    Insert,
    Delete,
}
#[derive(Event)]
pub struct StdbEvent {
    pub entity: Entity,
    pub node: TNode,
    pub change: StdbChange,
}

impl StdbPlugin {
    pub fn empty_queue_callback(
        world: &mut World,
        operation: impl FnOnce(&mut World) + Send + Sync + 'static,
    ) {
        world
            .resource_mut::<StdbData>()
            .on_empty
            .push(Box::new(operation));
    }
    fn unpack_node(node: &TNode, entity: Entity, world: &mut World) {
        debug!("Unpack {}#{entity}", node.kind);
        node.unpack(entity, world);
        world.send_event(StdbEvent {
            entity,
            node: node.clone(),
            change: StdbChange::Insert,
        });
    }
    fn update(world: &mut World) {
        world.resource_scope(|world, mut d: Mut<StdbData>| {
            let len = d.nodes_queue.len();
            d.nodes_queue.retain(|node| {
                if NODE_CONTAINERS.contains(&node.id) {
                    let entity = world.spawn_empty().id();
                    Self::unpack_node(node, entity, world);
                    return false;
                }
                let mut cur_node = node.clone();
                let mut id = node.id;
                loop {
                    let Some(parent) = cur_node.parent.get_node() else {
                        panic!("Parent of {cur_node:?} does not exist");
                    };
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
                let entity = world
                    .get_id_link(id)
                    .unwrap_or_else(|| world.spawn_empty().set_parent(parent).id());
                Self::unpack_node(node, entity, world);
                false
            });
            if len > 0 && d.nodes_queue.len() == len {
                debug!("Nodes queue stuck at {len}");
            }
            if len == 0 && !d.on_empty.is_empty() {
                let v = std::mem::take(&mut d.on_empty);
                for operation in v {
                    operation(world);
                }
            }
        });
    }
}

pub fn subscribe_game(on_success: impl FnOnce() + Send + Sync + 'static) {
    info!("Apply stdb subscriptions");
    cn().subscription_builder()
        .on_error(|_, error| error.to_string().notify_error_op())
        .on_applied(move |_| {
            info!("Subscription applied");
            on_success();
            subscribe_table_updates();
            op(|world| {
                let q = &mut world.resource_mut::<StdbData>().nodes_queue;
                for node in cn().db.nodes_world().iter() {
                    q.push(node);
                }
            });
        })
        .subscribe_to_all_tables();
}

fn subscribe_table_updates() {
    let db = cn().db();
    db.nodes_world().on_insert(|_, node| {
        on_insert(node);
    });
    db.nodes_world().on_update(|_, _, node| {
        on_update(node);
    });
    db.nodes_world().on_delete(|_, node| {
        on_delete(node);
    });
    db.battle().on_insert(|_, _| {
        todo!();
    });
}

fn on_insert(node: &TNode) {
    info!("Node inserted {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        world.resource_mut::<StdbData>().nodes_queue.push(node);
    });
}

fn on_delete(node: &TNode) {
    let node = node.clone();
    op(move |world| {
        let Some(entity) = world.get_id_link(node.id) else {
            error!("Failed to delete entity: id link not found");
            return;
        };
        info!("Node deleted {}#{} e:{entity}", node.kind, node.id);
        let id = node.id;
        let kind = node.kind();
        world.send_event(StdbEvent {
            entity,
            node,
            change: StdbChange::Delete,
        });
        kind.remove_component(entity, world);
        world.clear_id_link(id);
    });
}

fn on_update(node: &TNode) {
    info!("Node updated {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        let Some(entity) = world.get_id_link(node.id) else {
            error!("Failed to update entity: id link not found");
            return;
        };
        node.unpack(entity, world);
        world.send_event(StdbEvent {
            entity,
            node,
            change: StdbChange::Update,
        });
    });
}

pub fn subscribe_reducers() {
    cn().reducers.on_match_insert(|e| {
        e.event.notify_error();
    });
    cn().reducers.on_match_edit_fusion(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_buy(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_buy_fusion(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_start_battle(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|| {
            op(|world| {
                StdbPlugin::empty_queue_callback(world, |world| {
                    MatchPlugin::check_active(world).notify(world);
                    MatchPlugin::check_battles(world).notify(world);
                });
            });
        });
    });
    cn().reducers.on_match_submit_battle_result(|e, _, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|| {
            op(|world| {
                StdbPlugin::empty_queue_callback(world, |world| {
                    GameState::Shop.set_next(world);
                });
            });
        });
    });

    cn().reducers.on_incubator_vote(|e, _, _| {
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
