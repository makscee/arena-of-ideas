use bevy::ecs::event::{Event, EventReader, Events};
use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<StdbEvent>>();
        app.add_systems(Update, Self::handle_stdb_events);
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

impl StdbPlugin {
    fn handle_stdb_events(mut events: EventReader<StdbEvent>, state: Res<State<GameState>>) {
        for event in events.read() {
            Self::process_stdb_event(event, state.get());
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
                    // Use op() to schedule the cache reload
                    op(|world| {
                        crate::ui::NodeExplorerPlugin::load_kinds(world);
                    });
                }
            }
        }
    }

    /// Subscribe to StdbEvents while in a specific GameState
    ///
    /// # Examples
    /// ```
    /// // Subscribe to all events in the Explorer state
    /// let subscriber = StdbPlugin::subscribe_in_state(
    ///     GameState::Explorer,
    ///     |_event| true
    /// );
    ///
    /// // Subscribe to specific node types
    /// let subscriber = StdbPlugin::subscribe_in_state(
    ///     GameState::Explorer,
    ///     |event| event.node.kind == "NUnit"
    /// );
    /// ```
    pub fn subscribe_in_state<F>(state: GameState, filter: F) -> impl Fn(&StdbEvent, &GameState)
    where
        F: Fn(&StdbEvent) -> bool + Clone + 'static,
    {
        move |event, current_state| {
            if *current_state == state && filter(event) {
                Self::process_stdb_event(event, current_state);
            }
        }
    }

    /// Subscribe to StdbEvents for nodes with specific owners
    ///
    /// # Examples
    /// ```
    /// // Subscribe to core content changes (owner 0 and ID_CORE) in Explorer
    /// let subscriber = StdbPlugin::subscribe_owner_changes(
    ///     GameState::Explorer,
    ///     vec![0, ID_CORE]
    /// );
    ///
    /// // Subscribe to player-specific changes
    /// let subscriber = StdbPlugin::subscribe_owner_changes(
    ///     GameState::Shop,
    ///     vec![player_id()]
    /// );
    /// ```
    pub fn subscribe_owner_changes(
        state: GameState,
        owners: Vec<u64>,
    ) -> impl Fn(&StdbEvent, &GameState) {
        move |event, current_state| {
            if *current_state == state && owners.contains(&event.node.owner) {
                Self::process_stdb_event(event, current_state);
            }
        }
    }

    /// Subscribe to StdbEvents for specific node changes
    ///
    /// # Examples
    /// ```
    /// // Subscribe to unit insertions only
    /// let subscriber = StdbPlugin::subscribe_node_changes(
    ///     GameState::Explorer,
    ///     |event| event.node.kind == "NUnit",
    ///     |event| matches!(event.change, StdbChange::Insert)
    /// );
    /// ```
    pub fn subscribe_node_changes<F, C>(
        state: GameState,
        node_filter: F,
        change_filter: C,
    ) -> impl Fn(&StdbEvent, &GameState)
    where
        F: Fn(&StdbEvent) -> bool + Clone + 'static,
        C: Fn(&StdbEvent) -> bool + Clone + 'static,
    {
        move |event, current_state| {
            if *current_state == state && node_filter(event) && change_filter(event) {
                Self::process_stdb_event(event, current_state);
            }
        }
    }
}

// Usage examples for the StdbEvent subscription system:
//
// 1. Basic subscription for Explorer state:
// ```
// StdbPlugin::subscribe_in_state(GameState::Explorer, |event| {
//     event.node.kind == "NUnit" && event.node.owner == 0
// });
// ```
//
// 2. Subscribe to core content changes:
// ```
// StdbPlugin::subscribe_owner_changes(GameState::Explorer, vec![0, ID_CORE]);
// ```
//
// 3. Subscribe to specific node and change types:
// ```
// StdbPlugin::subscribe_node_changes(
//     GameState::Explorer,
//     |event| event.node.kind == "NHouse",
//     |event| matches!(event.change, StdbChange::Insert | StdbChange::Delete)
// );
// ```

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

        // Send StdbEvent
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
