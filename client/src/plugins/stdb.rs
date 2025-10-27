use super::*;
use crate::plugins::explorer::ExplorerState;

#[derive(Resource, Default)]
pub struct EventQueue {
    pending_events: VecDeque<QueuedEvent>,
}

#[derive(Clone)]
enum QueuedEvent {
    Node(StdbNodeEvent),
    Link(StdbLinkEvent),
}

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Messages<StdbNodeEvent>>();
        app.init_resource::<Messages<StdbLinkEvent>>();
        app.init_resource::<EventQueue>();
        app.add_systems(
            Update,
            (
                Self::handle_stdb_events,
                Self::handle_stdb_link_events,
                Self::process_event_queue,
            )
                .chain(),
        );
    }
}

#[derive(Clone)]
pub enum StdbChange {
    Update,
    Insert,
    Delete,
}

#[derive(Message, Clone)]
pub struct StdbNodeEvent {
    pub node: TNode,
    pub change: StdbChange,
}

#[derive(Message, Clone)]
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
    fn handle_stdb_events(mut events: MessageReader<StdbNodeEvent>, state: Res<State<GameState>>) {
        for event in events.read() {
            Self::process_stdb_event(event, state.get());
        }
    }

    fn handle_stdb_link_events(
        mut events: MessageReader<StdbLinkEvent>,
        state: Res<State<GameState>>,
    ) {
        for event in events.read() {
            Self::process_stdb_link_event(event, state.get());
        }
    }

    fn process_event_queue(mut queue: ResMut<EventQueue>) {
        let mut processed = 0;
        let initial_len = queue.pending_events.len();

        while processed < initial_len {
            if let Some(event) = queue.pending_events.pop_front() {
                match event {
                    QueuedEvent::Node(node_event) => {
                        Self::process_queued_node_event(&node_event);
                    }
                    QueuedEvent::Link(link_event) => {
                        Self::try_process_queued_link_event(&link_event);
                    }
                }
            }
            processed += 1;
        }
    }

    fn try_process_queued_link_event(link_event: &StdbLinkEvent) {
        let event_clone = link_event.clone();
        op(move |world| {
            let can_process = with_solid_source(|ctx| {
                ctx.entity(event_clone.parent)?;
                ctx.entity(event_clone.child)?;
                Ok(())
            })
            .is_ok();

            if can_process {
                Self::process_queued_link_event_inner(world, &event_clone);
            } else {
                // Re-queue the event if nodes aren't ready yet
                if let Some(mut queue) = world.get_resource_mut::<EventQueue>() {
                    queue
                        .pending_events
                        .push_back(QueuedEvent::Link(event_clone));
                }
            }
        });
    }

    fn process_queued_node_event(node_event: &StdbNodeEvent) {
        let node_event = node_event.clone();

        // Only process nodes with owner_id = 0 or 1 for static sources
        if node_event.node.owner == 0 || node_event.node.owner == 1 {
            let stdb_update = match node_event.change {
                StdbChange::Insert => StdbUpdate::NodeInsert(node_event.node.clone()),
                StdbChange::Update => StdbUpdate::NodeUpdate {
                    old: node_event.node.clone(),
                    new: node_event.node.clone(),
                },
                StdbChange::Delete => StdbUpdate::NodeDelete(node_event.node.clone()),
            };

            with_static_sources(|sources| {
                sources.solid.handle_stdb_update(&stdb_update);
                sources.top.handle_stdb_update(&stdb_update);
                sources.selected.handle_stdb_update(&stdb_update);
            });
        }

        match node_event.change {
            StdbChange::Insert => {
                op(move |world| {
                    with_solid_source(|ctx| {
                        node_event.node.id.kind_db()?.spawn(ctx, &node_event.node)?;
                        Ok(())
                    })
                    .log();
                });
            }
            StdbChange::Update => {
                op(move |world| {
                    with_solid_source(|ctx| {
                        node_event.node.id.kind_db()?.spawn(ctx, &node_event.node)?;
                        Ok(())
                    })
                    .log();
                });
            }
            StdbChange::Delete => {
                op(move |world| {
                    with_selected_source(|ctx| {
                        node_event.node.id.kind_db()?.spawn(ctx, &node_event.node)?;
                        Ok(())
                    })
                    .log();
                });
            }
        }
    }

    fn process_queued_link_event_inner(world: &mut World, link_event: &StdbLinkEvent) {
        // Pass link events to static sources for nodes with owner_id = 0 or 1
        let stdb_update = match link_event.change {
            StdbChange::Insert => {
                if link_event.solid {
                    StdbUpdate::LinkInsert(TNodeLink {
                        id: 0, // ID not used in updates
                        parent: link_event.parent,
                        child: link_event.child,
                        parent_kind: link_event.parent_kind.clone(),
                        child_kind: link_event.child_kind.clone(),
                        rating: link_event.rating,
                        solid: link_event.solid,
                    })
                } else {
                    return;
                }
            }
            StdbChange::Update => {
                if link_event.solid {
                    StdbUpdate::LinkInsert(TNodeLink {
                        id: 0, // ID not used in updates
                        parent: link_event.parent,
                        child: link_event.child,
                        parent_kind: link_event.parent_kind.clone(),
                        child_kind: link_event.child_kind.clone(),
                        rating: link_event.rating,
                        solid: link_event.solid,
                    })
                } else {
                    StdbUpdate::LinkDelete(TNodeLink {
                        id: 0, // ID not used in updates
                        parent: link_event.parent,
                        child: link_event.child,
                        parent_kind: link_event.parent_kind.clone(),
                        child_kind: link_event.child_kind.clone(),
                        rating: link_event.rating,
                        solid: true,
                    })
                }
            }
            StdbChange::Delete => StdbUpdate::LinkDelete(TNodeLink {
                id: 0, // ID not used in updates
                parent: link_event.parent,
                child: link_event.child,
                parent_kind: link_event.parent_kind.clone(),
                child_kind: link_event.child_kind.clone(),
                rating: link_event.rating,
                solid: link_event.solid,
            }),
        };

        with_static_sources(|sources| {
            sources.solid.handle_stdb_update(&stdb_update);
            sources.top.handle_stdb_update(&stdb_update);
            sources.selected.handle_stdb_update(&stdb_update);
        });

        world.write_message(link_event.clone());
    }

    fn process_stdb_event(event: &StdbNodeEvent, current_state: &GameState) {
        // Handle Explorer cache refresh for node changes
        if *current_state == GameState::Explorer
            && (event.node.owner == 0 || event.node.owner == ID_CORE)
        {
            match event.change {
                StdbChange::Insert | StdbChange::Update | StdbChange::Delete => {
                    // Refresh Explorer cache when content nodes change
                    op(move |world| {
                        if let Some(mut explorer_cache) = world.get_resource_mut::<ExplorerCache>()
                        {
                            ExplorerState::refresh_cache(&mut explorer_cache);
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
                        if let Some(mut explorer_cache) = world.get_resource_mut::<ExplorerCache>()
                        {
                            ExplorerState::refresh_cache(&mut explorer_cache);
                        }
                    });
                }
            }
        }
    }
}

pub fn subscribe_game(on_success: impl FnOnce() + Send + Sync + 'static) {
    info!("Apply stdb subscriptions");
    init_static_sources();
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
            let link_event = StdbLinkEvent {
                parent,
                child,
                parent_kind,
                child_kind,
                rating,
                solid,
                change: StdbChange::Insert,
            };
            if let Some(mut queue) = world.get_resource_mut::<EventQueue>() {
                queue
                    .pending_events
                    .push_back(QueuedEvent::Link(link_event));
            }
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
            let link_event = StdbLinkEvent {
                parent,
                child,
                parent_kind,
                child_kind,
                rating,
                solid,
                change: StdbChange::Update,
            };
            if let Some(mut queue) = world.get_resource_mut::<EventQueue>() {
                queue
                    .pending_events
                    .push_back(QueuedEvent::Link(link_event));
            }
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
            let link_event = StdbLinkEvent {
                parent,
                child,
                parent_kind,
                child_kind,
                rating,
                solid,
                change: StdbChange::Delete,
            };
            if let Some(mut queue) = world.get_resource_mut::<EventQueue>() {
                queue
                    .pending_events
                    .push_back(QueuedEvent::Link(link_event));
            }
        });
    });
}

fn on_insert(node: &TNode) {
    info!("Node inserted {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        let node_event = StdbNodeEvent {
            node,
            change: StdbChange::Insert,
        };
        if let Some(mut queue) = world.get_resource_mut::<EventQueue>() {
            queue
                .pending_events
                .push_back(QueuedEvent::Node(node_event));
        }
    });
}

fn on_delete(node: &TNode) {
    let node = node.clone();
    op(move |world| {
        let node_event = StdbNodeEvent {
            node,
            change: StdbChange::Delete,
        };
        if let Some(mut queue) = world.get_resource_mut::<EventQueue>() {
            queue
                .pending_events
                .push_back(QueuedEvent::Node(node_event));
        }
    });
}

fn on_update(node: &TNode) {
    info!("Node updated {}#{}", node.kind, node.id);
    let node = node.clone();
    op(move |world| {
        let node_event = StdbNodeEvent {
            node,
            change: StdbChange::Update,
        };
        if let Some(mut queue) = world.get_resource_mut::<EventQueue>() {
            queue
                .pending_events
                .push_back(QueuedEvent::Node(node_event));
        }
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
    cn().reducers.on_admin_upload_world(|e, _, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
}
