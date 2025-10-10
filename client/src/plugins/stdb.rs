use bevy::ecs::event::{Event, EventReader, Events};
use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;
use crate::plugins::explorer::ExplorerState;

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<StdbEvent>>();
        app.init_resource::<Events<StdbLinkEvent>>();
        app.add_systems(
            Update,
            (Self::handle_stdb_events, Self::handle_stdb_link_events),
        );
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

#[derive(Event)]
pub struct StdbLinkEvent {
    pub parent: u64,
    pub child: u64,
    pub parent_kind: String,
    pub child_kind: String,
    pub rating: i32,
    pub solid: bool,
    pub change: StdbChange,
}

impl StdbPlugin {
    fn handle_stdb_events(mut events: EventReader<StdbEvent>, state: Res<State<GameState>>) {
        for event in events.read() {
            Self::process_stdb_event(event, state.get());
        }
    }

    fn handle_stdb_link_events(
        mut events: EventReader<StdbLinkEvent>,
        state: Res<State<GameState>>,
    ) {
        for event in events.read() {
            Self::process_stdb_link_event(event, state.get());
        }
    }

    fn process_stdb_event(event: &StdbEvent, current_state: &GameState) {
        // Handle Explorer cache refresh for node changes
        if *current_state == GameState::Explorer
            && (event.node.owner == 0 || event.node.owner == ID_CORE)
        {
            match event.change {
                StdbChange::Insert | StdbChange::Update | StdbChange::Delete => {
                    // Refresh Explorer cache when content nodes change
                    op(move |world| {
                        if let Some(mut explorer_state) = world.get_resource_mut::<ExplorerState>()
                        {
                            explorer_state.refresh_named_cache();
                        }
                    });
                }
            }
        }
    }

    fn process_stdb_link_event(event: &StdbLinkEvent, current_state: &GameState) {
        // Handle Explorer cache refresh for link changes
        if *current_state == GameState::Explorer {
            match event.change {
                StdbChange::Insert | StdbChange::Update | StdbChange::Delete => {
                    // Refresh Explorer cache when links change
                    op(move |world| {
                        if let Some(mut explorer_state) = world.get_resource_mut::<ExplorerState>()
                        {
                            explorer_state.refresh_named_cache();
                        }
                    });
                }
            }
        }
    }
}

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
        let parent_kind = link.parent_kind.clone();
        let child_kind = link.child_kind.clone();
        op(move |world| {
            if solid {
                world
                    .as_context_mut()
                    .add_link(parent, child)
                    .track()
                    .notify_error_op();
            }
            // world.set_link_rating(parent, child, rating, solid);
            world.send_event(StdbLinkEvent {
                parent,
                child,
                parent_kind,
                child_kind,
                rating,
                solid,
                change: StdbChange::Insert,
            });
        });
    });
    db.node_links().on_update(|_, _, link| {
        debug!("update link {link:?}");
        let parent = link.parent;
        let child = link.child;
        let rating = link.rating;
        let solid = link.solid;
        let parent_kind = link.parent_kind.clone();
        let child_kind = link.child_kind.clone();
        op(move |world| {
            if solid {
                world
                    .as_context_mut()
                    .add_link(parent, child)
                    .track()
                    .notify_error_op();
            } else {
                world
                    .as_context_mut()
                    .remove_link(parent, child)
                    .notify_error_op();
            }
            // world.set_link_rating(parent, child, rating, solid);
            world.send_event(StdbLinkEvent {
                parent,
                child,
                parent_kind,
                child_kind,
                rating,
                solid,
                change: StdbChange::Update,
            });
        });
    });
    db.node_links().on_delete(|_, link| {
        debug!("remove link {link:?}");
        let parent = link.parent;
        let child = link.child;
        let rating = link.rating;
        let solid = link.solid;
        let parent_kind = link.parent_kind.clone();
        let child_kind = link.child_kind.clone();
        op(move |world| {
            if solid {
                world
                    .as_context()
                    .remove_link(parent, child)
                    .notify_error_op();
            }
            world.send_event(StdbLinkEvent {
                parent,
                child,
                parent_kind,
                child_kind,
                rating,
                solid,
                change: StdbChange::Delete,
            });
        });
    });
}

fn on_insert(node: &TNode) {
    info!("Node inserted {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        let entity = world.spawn_empty().id();
        world
            .with_context(|ctx| node.kind().spawn(ctx, &node))
            .notify_error_op();
        world.send_event(StdbEvent {
            entity,
            node,
            change: StdbChange::Insert,
        });
    });
}

fn on_delete(node: &TNode) {
    let node = node.clone();
    op(move |world| {
        world
            .with_context(|ctx| {
                let entity = ctx.entity(node.id)?;
                info!("Node deleted {}#{} e:{entity}", node.kind, node.id);
                ctx.world_mut()?.send_event(StdbEvent {
                    entity,
                    node: node.clone(),
                    change: StdbChange::Delete,
                });
                ctx.despawn(node.id)
            })
            .log();
    });
}

fn on_update(node: &TNode) {
    info!("Node updated {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        world
            .with_context(|ctx| {
                let entity = ctx.entity(node.id)?;
                node.kind().spawn(ctx, &node)?;
                ctx.world_mut()?.send_event(StdbEvent {
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
