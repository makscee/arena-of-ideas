use bevy::ecs::event::{Event, Events};
use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<StdbEvent>>();
    }
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

impl StdbPlugin {}

pub fn subscribe_game(on_success: impl FnOnce() + Send + Sync + 'static) {
    info!("Apply stdb subscriptions");
    subscribe_table_updates();
    cn().subscription_builder()
        .on_error(|_, error| error.to_string().notify_error_op())
        .on_applied(move |_| {
            info!("Subscription applied");
            on_success();
        })
        .subscribe_to_all_tables();
}

fn subscribe_table_updates() {
    let db = cn().db();
    db.nodes_world().on_insert(|_, node| {
        debug!("insert node {node:?}");
        on_insert(node);
    });
    db.nodes_world().on_update(|_, _, node| {
        on_update(node);
    });
    db.nodes_world().on_delete(|_, node| {
        on_delete(node);
    });

    db.node_links().on_insert(|_, link| {
        debug!("add link {link:?}");
        let parent = link.parent;
        let child = link.child;
        let rating = link.rating;
        let solid = link.solid;
        op(move |world| {
            if solid {
                world.link_parent_child(parent, child);
            }
            world.set_link_rating(parent, child, rating, solid);
        });
    });
    db.node_links().on_update(|_, _, link| {
        debug!("update link {link:?}");
        let parent = link.parent;
        let child = link.child;
        let rating = link.rating;
        let solid = link.solid;
        op(move |world| {
            if solid {
                world.link_parent_child(parent, child);
            } else {
                world.unlink_parent_child(parent, child);
            }
            world.set_link_rating(parent, child, rating, solid);
        });
    });
    db.node_links().on_delete(|_, link| {
        debug!("remove link {link:?}");
        let parent = link.parent;
        let child = link.child;
        if link.solid {
            op(move |world| {
                world.unlink_parent_child(parent, child);
            });
        }
    });
}

fn on_insert(node: &TNode) {
    info!("Node inserted {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        let entity = world.spawn_empty().id();
        Context::from_world(world, |context| {
            node.unpack(context, entity);
        });
    });
}

fn on_delete(node: &TNode) {
    let node = node.clone();
    op(move |world| {
        Context::from_world_r(world, |context| {
            let entity = context.entity(node.id)?;
            info!("Node deleted {}#{} e:{entity}", node.kind, node.id);
            context.world_mut()?.send_event(StdbEvent {
                entity,
                node,
                change: StdbChange::Delete,
            });
            context.despawn(entity)
        })
        .log();
    });
}

fn on_update(node: &TNode) {
    info!("Node updated {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        Context::from_world_r(world, |context| {
            let entity = context.entity(node.id)?;
            node.unpack(context, entity);
            context.world_mut()?.send_event(StdbEvent {
                entity,
                node,
                change: StdbChange::Update,
            });
            Ok(())
        })
        .log();
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
    cn().reducers.on_match_add_fusion_unit(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_remove_fusion_unit(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_reorder_fusions(|e, _| {
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
    cn().reducers.on_match_play_unit(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_play_house(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_buy_fusion_lvl(|e, _| {
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
    cn().reducers.on_match_submit_battle_result(|e, _, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.on_success(|| {
            op(|world| {
                GameState::Shop.set_next(world);
            });
        });
    });
    cn().reducers.on_admin_delete_node(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_admin_upload_world(|e, _, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
}
