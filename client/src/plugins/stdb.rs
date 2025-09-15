use bevy::ecs::event::{Event, EventReader, Events};
use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

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
        // Handle Explorer cache refresh for nodes with owner 0 or 1
        if (*current_state == GameState::Explorer)
            && (event.node.owner == 0 || event.node.owner == ID_CORE)
        {
            match event.change {
                StdbChange::Insert | StdbChange::Update | StdbChange::Delete => {
                    info!(
                        "Reloading Explorer cache due to node change: {}#{}",
                        event.node.kind, event.node.id
                    );
                    op(|world| {
                        crate::ui::NodeExplorerPlugin::load_kinds(world);
                    });
                }
            }
        }

        // Handle ContentExplorer cache refresh for node changes
        if *current_state == GameState::ContentExplorer
            && (event.node.owner == 0 || event.node.owner == ID_CORE)
        {
            match event.change {
                StdbChange::Insert | StdbChange::Update | StdbChange::Delete => {
                    info!(
                        "Reloading ContentExplorer cache due to node change: {}#{}",
                        event.node.kind, event.node.id
                    );
                    op(|world| {
                        if let Some(mut data) = world.get_resource_mut::<ContentExplorerData>() {
                            data.needs_refresh = true;
                        }
                    });
                }
            }
        }
    }

    fn process_stdb_link_event(event: &StdbLinkEvent, current_state: &GameState) {
        // Handle ContentExplorer cache refresh for link changes
        if *current_state == GameState::ContentExplorer {
            match event.change {
                StdbChange::Insert | StdbChange::Update | StdbChange::Delete => {
                    info!(
                        "Reloading ContentExplorer cache due to link change: {}->{}",
                        event.parent, event.child
                    );
                    op(|world| {
                        if let Some(mut data) = world.get_resource_mut::<ContentExplorerData>() {
                            data.needs_refresh = true;
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
                world.link_parent_child(parent, child);
            }
            world.set_link_rating(parent, child, rating, solid);
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
                world.link_parent_child(parent, child);
            } else {
                world.unlink_parent_child(parent, child);
            }
            world.set_link_rating(parent, child, rating, solid);
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
                world.unlink_parent_child(parent, child);
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
        Context::from_world(world, |context| {
            node.unpack(context, entity);
        });
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
        Context::from_world_r(world, |context| {
            let entity = context.entity(node.id)?;
            info!("Node deleted {}#{} e:{entity}", node.kind, node.id);
            context.world_mut()?.send_event(StdbEvent {
                entity,
                node: node.clone(),
                change: StdbChange::Delete,
            });
            context.despawn(entity)
        })
        .log();

        // Send StdbEvent at world level as well
        world.send_event(StdbEvent {
            entity: Entity::PLACEHOLDER,
            node,
            change: StdbChange::Delete,
        });
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
                node: node.clone(),
                change: StdbChange::Update,
            });
            Ok(())
        })
        .log();

        // Send StdbEvent at world level as well
        world.send_event(StdbEvent {
            entity: Entity::PLACEHOLDER,
            node,
            change: StdbChange::Update,
        });
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
