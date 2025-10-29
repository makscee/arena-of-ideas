use super::*;

#[derive(Resource, Default)]
pub struct UpdateQueue {
    pending_updates: VecDeque<StdbUpdate>,
    failed_updates: VecDeque<StdbUpdate>,
    new_updates_since_retry: bool,
}

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpdateQueue>();
        app.add_systems(Update, Self::process_update_queue);
    }
}

impl StdbPlugin {
    fn process_update_queue(mut queue: ResMut<UpdateQueue>) {
        // Process new pending updates
        let mut processed_successfully = Vec::new();
        let mut failed_updates = Vec::new();

        while let Some(update) = queue.pending_updates.pop_front() {
            let mut all_succeeded = true;
            with_static_sources(|sources| {
                if sources.solid.handle_stdb_update(&update).is_err() {
                    all_succeeded = false;
                }
                if sources.top.handle_stdb_update(&update).is_err() {
                    all_succeeded = false;
                }
                if sources.selected.handle_stdb_update(&update).is_err() {
                    all_succeeded = false;
                }
            });

            if all_succeeded {
                processed_successfully.push(update);
            } else {
                failed_updates.push(update);
            }
        }

        // Retry failed updates only if new updates were processed
        if queue.new_updates_since_retry && !queue.failed_updates.is_empty() {
            let mut retry_failed = Vec::new();

            while let Some(update) = queue.failed_updates.pop_front() {
                let mut all_succeeded = true;

                with_static_sources(|sources| {
                    if sources.solid.handle_stdb_update(&update).is_err() {
                        all_succeeded = false;
                    }
                    if sources.top.handle_stdb_update(&update).is_err() {
                        all_succeeded = false;
                    }
                    if sources.selected.handle_stdb_update(&update).is_err() {
                        all_succeeded = false;
                    }
                });

                if !all_succeeded {
                    retry_failed.push(update);
                }
            }

            queue.failed_updates.extend(retry_failed);
            queue.new_updates_since_retry = false;
        }

        // Add new failed updates to the failed queue
        queue.failed_updates.extend(failed_updates);
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
        queue_update(StdbUpdate::NodeInsert(node.clone()));
    });

    db.nodes_world().on_update(|_, old, new| {
        debug!("update node {new:?}");
        queue_update(StdbUpdate::NodeUpdate {
            old: old.clone(),
            new: new.clone(),
        });
    });

    db.nodes_world().on_delete(|_, node| {
        debug!("delete node {node:?}");
        queue_update(StdbUpdate::NodeDelete(node.clone()));
    });

    db.node_links().on_insert(|_, link| {
        debug!("insert link {link:?}");
        queue_update(StdbUpdate::LinkInsert(link.clone()));
    });

    db.node_links().on_update(|_, old, new| {
        debug!("update link {new:?}");
        queue_update(StdbUpdate::LinkUpdate {
            old: old.clone(),
            new: new.clone(),
        });
    });

    db.node_links().on_delete(|_, link| {
        debug!("delete link {link:?}");
        if link.solid {
            queue_update(StdbUpdate::LinkDelete(link.clone()));
        }
    });
}

fn queue_update(update: StdbUpdate) {
    op(move |world| {
        if let Some(mut queue) = world.get_resource_mut::<UpdateQueue>() {
            queue.pending_updates.push_back(update);
            queue.new_updates_since_retry = true;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_update_queue_basic() {
        let mut queue = UpdateQueue::default();
        let node = NHouse::new(1001, "house name".into()).to_tnode();

        let update = StdbUpdate::NodeInsert(node);
        queue.pending_updates.push_back(update);

        assert_eq!(queue.pending_updates.len(), 1);

        let updates: Vec<StdbUpdate> = queue.pending_updates.drain(..).collect();
        assert_eq!(updates.len(), 1);
        assert_eq!(queue.pending_updates.len(), 0);
    }

    #[test]
    fn test_node_creation_and_retrieval() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, "house name".into()).to_tnode();
        let update = StdbUpdate::NodeInsert(house_node);
        solid_source.handle_stdb_update(&update).unwrap();
        let entity = solid_source.entity(1001).expect("Node entity should exist");
        let world = solid_source.world().expect("World should be accessible");
        assert!(
            world.get::<NHouse>(entity).is_some(),
            "NHouse component should exist"
        );
        let house = world.get::<NHouse>(entity).unwrap();
        assert_eq!(house.id, 1001);
        assert_eq!(house.house_name, "house name");
    }

    #[test]
    fn test_component_entity_merging() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let color_node = NHouseColor::new(1002, default()).to_tnode();
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();
        let house_entity = solid_source
            .entity(1001)
            .expect("House entity should exist");
        let color_entity = solid_source
            .entity(1002)
            .expect("Color entity should exist");
        assert_eq!(
            house_entity, color_entity,
            "House and Color should be on same entity"
        );
        let world = solid_source.world().expect("World should be accessible");
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "NHouse should exist on entity"
        );
        assert!(
            world.get::<NHouseColor>(house_entity).is_some(),
            "NHouseColor should exist on entity"
        );
    }

    #[test]
    fn test_component_entity_merging_nonsolid() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let color_node = NHouseColor::new(1002, default()).to_tnode();
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 0,
            solid: false,
        };
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();
        let house_entity = top_source.entity(1001).expect("House entity should exist");
        let color_entity = top_source.entity(1002).expect("Color entity should exist");
        assert_eq!(
            house_entity, color_entity,
            "House and Color should be on same entity"
        );
        let world = top_source.world().expect("World should be accessible");
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "NHouse should exist on entity"
        );
        assert!(
            world.get::<NHouseColor>(house_entity).is_some(),
            "NHouseColor should exist on entity"
        );
    }

    #[test]
    fn test_top_source_ratings_and_merging() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let color_node1 = NHouseColor::new(1002, default()).to_tnode();
        let color_node2 = NHouseColor::new(1003, default()).to_tnode();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node1))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node2))
            .unwrap();

        let link1 = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 5,
            solid: false,
        };
        let link2 = TNodeLink {
            id: 2,
            parent: 1001,
            child: 1003,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 10, // Higher rating
            solid: false,
        };

        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link1))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link2))
            .unwrap();
        let house_entity = top_source.entity(1001).unwrap();
        let higher_rated_entity = top_source.entity(1003).unwrap();
        assert_eq!(
            house_entity, higher_rated_entity,
            "Higher rated child should be merged with parent"
        );
        let children = top_source
            .get_children_of_kind(1001, NodeKind::NHouseColor)
            .unwrap();
        assert!(
            children.contains(&1003),
            "Should contain higher rated child"
        );
        assert!(
            !children.contains(&1002),
            "Should not contain lower rated child"
        );
    }

    #[test]
    fn test_selected_source_player_selection() {
        let mut selected_source = Sources::new_selected();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let unit_node = NUnit::new(1002, default()).to_tnode();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node))
            .unwrap();
        let selection = TPlayerLinkSelection {
            id: 1,
            player_id: player_id(),
            parent_id: 1001,
            kind: "NUnit".to_string(),
            selected_link_id: 1002,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionInsert(selection))
            .unwrap();
        let children = selected_source
            .get_children_of_kind(1001, NodeKind::NUnit)
            .unwrap();
        assert!(
            children.contains(&1002),
            "Selected source should contain player's selection"
        );
    }

    #[test]
    fn test_selected_source_player_selection_update() {
        let mut selected_source = Sources::new_selected();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let unit_node1 = NUnit::new(1002, default()).to_tnode();
        let unit_node2 = NUnit::new(1003, default()).to_tnode();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node1))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node2))
            .unwrap();
        let old_selection = TPlayerLinkSelection {
            id: 1,
            player_id: player_id(),
            parent_id: 1001,
            kind: "NUnit".to_string(),
            selected_link_id: 1002,
        };
        let new_selection = TPlayerLinkSelection {
            id: 1,
            player_id: player_id(),
            parent_id: 1001,
            kind: "NUnit".to_string(),
            selected_link_id: 1003,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionInsert(
                old_selection.clone(),
            ))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionUpdate {
                old: old_selection,
                new: new_selection,
            })
            .unwrap();
        let children = selected_source
            .get_children_of_kind(1001, NodeKind::NUnit)
            .unwrap();
        assert!(children.contains(&1003), "Should contain new selection");
        assert!(
            !children.contains(&1002),
            "Should not contain old selection"
        );
    }

    #[test]
    fn test_selected_source_ignores_other_players() {
        let mut selected_source = Sources::new_selected();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let unit_node = NUnit::new(1002, default()).to_tnode();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node))
            .unwrap();
        let other_player_selection = TPlayerLinkSelection {
            id: 1,
            player_id: player_id() + 1, // Different player
            parent_id: 1001,
            kind: "NUnit".to_string(),
            selected_link_id: 1002,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionInsert(
                other_player_selection,
            ))
            .unwrap();
        let children = selected_source
            .get_children_of_kind(1001, NodeKind::NUnit)
            .unwrap_or_default();
        assert!(
            !children.contains(&1002),
            "Selected source should ignore other player's selections"
        );
    }

    #[test]
    fn test_top_source_link_deletion_and_rating_update() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let color_node1 = NHouseColor::new(1002, default()).to_tnode();
        let color_node2 = NHouseColor::new(1003, default()).to_tnode();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node1))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node2))
            .unwrap();
        let link1 = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 5,
            solid: false,
        };
        let link2 = TNodeLink {
            id: 2,
            parent: 1001,
            child: 1003,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 10,
            solid: false,
        };
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link1))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link2.clone()))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::LinkDelete(link2))
            .unwrap();
        let children = top_source
            .get_children_of_kind(1001, NodeKind::NHouseColor)
            .unwrap_or_default();
        assert!(
            children.contains(&1002),
            "Should fall back to lower rated child after deletion"
        );
        assert!(!children.contains(&1003), "Should not contain deleted link");
    }

    #[test]
    fn test_reverse_order_component_merging() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, "house name".into()).to_tnode();
        let color_node = NHouseColor::new(1002, default()).to_tnode();
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        let house_entity = solid_source
            .entity(1001)
            .expect("House entity should exist");
        let color_entity = solid_source
            .entity(1002)
            .expect("Color entity should exist");
        assert_eq!(house_entity, color_entity, "Entities should be merged");
        let world = solid_source.world().expect("World should be accessible");
        let house = world.get::<NHouse>(house_entity).unwrap();
        let color = world.get::<NHouseColor>(house_entity).unwrap();
        assert_eq!(house.house_name, "house name");
        assert_eq!(color.color.0, "#ffffff");
    }

    #[test]
    fn test_link_processing() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let color_node = NHouseColor::new(1002, default()).to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link.clone()))
            .unwrap();
        let children = solid_source
            .get_children_of_kind(1001, NodeKind::NHouseColor)
            .unwrap();
        assert!(
            children.contains(&1002),
            "Link should exist from house to color"
        );
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkDelete(link))
            .unwrap();
        let children_after = solid_source
            .get_children_of_kind(1001, NodeKind::NHouseColor)
            .unwrap();
        assert!(!children_after.contains(&1002), "Link should be removed");
    }

    #[test]
    fn test_node_update_preserves_entity() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, "house name".into()).to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node.clone()))
            .unwrap();
        let original_entity = solid_source.entity(1001).unwrap();
        let updated_node = NHouse::new(1001, "house name 2".into()).to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeUpdate {
                old: house_node,
                new: updated_node,
            })
            .unwrap();
        let updated_entity = solid_source.entity(1001).unwrap();
        assert_eq!(
            original_entity, updated_entity,
            "Entity should remain the same after update"
        );
        let world = solid_source.world().unwrap();
        let house = world.get::<NHouse>(updated_entity).unwrap();
        assert_eq!(house.house_name, "house name 2");
    }

    #[test]
    fn test_multiple_entity_merging() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, default()).to_tnode();
        let color_node = NHouseColor::new(1002, default()).to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();
        let house_entity = solid_source.entity(1001).unwrap();
        let color_entity = solid_source.entity(1002).unwrap();
        assert_ne!(
            house_entity, color_entity,
            "Should start on different entities"
        );
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();
        let ability_node = NAbilityMagic::new(1003, default()).to_tnode();
        let ability_link = TNodeLink {
            id: 2,
            parent: 1001,
            child: 1003,
            parent_kind: "NHouse".to_string(),
            child_kind: "NAbilityMagic".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(ability_link))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(ability_node))
            .unwrap();
        let final_house_entity = solid_source.entity(1001).unwrap();
        let final_color_entity = solid_source.entity(1002).unwrap();
        let final_ability_entity = solid_source.entity(1003).unwrap();
        assert_eq!(final_house_entity, final_color_entity);
        assert_eq!(final_house_entity, final_ability_entity);
        assert_eq!(final_color_entity, final_ability_entity);
        let world = solid_source.world().unwrap();
        assert!(world.get::<NHouse>(final_house_entity).is_some());
        assert!(world.get::<NHouseColor>(final_house_entity).is_some());
        assert!(world.get::<NAbilityMagic>(final_house_entity).is_some());
    }
}
