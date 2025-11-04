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

        // Check if this is the current player's NMatch and if pending_battle flag was set
        if new.kind() == NodeKind::NMatch && new.owner == player_id() {
            // Parse the data to check pending_battle field
            if let (Ok(old_match), Ok(new_match)) =
                (old.to_node::<NMatch>(), new.to_node::<NMatch>())
            {
                if !old_match.pending_battle && new_match.pending_battle {
                    // Battle became pending, trigger state transition
                    op(|world| {
                        if matches!(world.resource::<State<GameState>>().get(), GameState::Shop) {
                            GameState::Battle.set_next(world);
                        }
                    });
                }
            }
        }
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
        queue_update(StdbUpdate::LinkDelete(link.clone()));
    });

    db.player_link_selections().on_insert(|_, link| {
        debug!("insert player selection link {link:?}");
        queue_update(StdbUpdate::PlayerLinkSelectionInsert(link.clone()));
    });
    db.player_link_selections().on_update(|_, old, new| {
        debug!("update player selection link {new:?}");
        queue_update(StdbUpdate::PlayerLinkSelectionUpdate {
            old: old.clone(),
            new: new.clone(),
        });
    });
    db.player_link_selections().on_delete(|_, link| {
        debug!("delete link {link:?}");
        queue_update(StdbUpdate::PlayerLinkSelectionDelete(link.clone()));
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
    cn().reducers.on_match_shop_buy(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_shop_reroll(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_move_unit(|e, _, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_bench_unit(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_buy_fusion_slot(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_match_start_battle(|e| {
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
    cn().reducers.on_content_rotation(|e| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
    });
    cn().reducers.on_content_select_link(|e, _, _| {
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
    fn test_node_creation_and_retrieval() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, 0, "house name".into()).to_tnode();
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
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();
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
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();
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
            "House and Color should be on same entities in Top source"
        );
        let world = top_source.world().expect("World should be accessible");
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "NHouse should exist on entity"
        );
        assert!(
            world.get::<NHouseColor>(color_entity).is_some(),
            "NHouseColor should exist on entity"
        );
        let children = top_source
            .get_children_of_kind(1001, NodeKind::NHouseColor)
            .unwrap();
        assert!(
            children.contains(&1002),
            "House should have color child linked"
        );
    }

    #[test]
    fn test_top_source_ratings_and_merging() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node1 = NHouseColor::new(1002, 0, default()).to_tnode();
        let color_node2 = NHouseColor::new(1003, 0, default()).to_tnode();
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
        let color1_entity = top_source.entity(1002).unwrap();
        let color2_entity = top_source.entity(1003).unwrap();

        assert_ne!(
            house_entity, color1_entity,
            "First color should get extracted from house"
        );
        assert_eq!(
            house_entity, color2_entity,
            "Second color should be merged with house"
        );

        let world = top_source.world().unwrap();
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "House should exist on merged entity"
        );
        assert!(
            world.get::<NHouseColor>(house_entity).is_some(),
            "Color should exist on merged entity"
        );
        let children = top_source
            .get_children_of_kind(1001, NodeKind::NHouseColor)
            .unwrap();
        assert_eq!(
            children.len(),
            1,
            "Should only have one linked child (top-rated)"
        );
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
        set_player_id_for_test(999);
        let mut selected_source = Sources::new_selected();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let unit_node = NUnit::new(1002, 0, default()).to_tnode();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node))
            .unwrap();

        let link = TNodeLink {
            id: 5001,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 0,
            solid: true,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();

        let selection = TPlayerLinkSelection {
            id: 1,
            player_id: 999,
            parent_id: 1001,
            child_id: 1002,
            link_id: 5001,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionInsert(selection))
            .unwrap();
        let unit_entity = selected_source.entity(1002).unwrap();
        let world = selected_source.world().unwrap();
        assert!(
            world.get::<NUnit>(unit_entity).is_some(),
            "Unit should exist in world"
        );
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
        set_player_id_for_test(999);
        let mut selected_source = Sources::new_selected();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let unit_node1 = NUnit::new(1002, 0, default()).to_tnode();
        let unit_node2 = NUnit::new(1003, 0, default()).to_tnode();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node1))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node2))
            .unwrap();

        let link1 = TNodeLink {
            id: 5002,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 0,
            solid: true,
        };
        let link2 = TNodeLink {
            id: 5003,
            parent: 1001,
            child: 1003,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 0,
            solid: true,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link1))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link2))
            .unwrap();

        let old_selection = TPlayerLinkSelection {
            id: 1,
            player_id: 999,
            parent_id: 1001,
            child_id: 1002,
            link_id: 5002,
        };
        let new_selection = TPlayerLinkSelection {
            id: 1,
            player_id: 999,
            parent_id: 1001,
            child_id: 1003,
            link_id: 5003,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionInsert(
                old_selection.clone(),
            ))
            .unwrap();
        let unit1_entity = selected_source.entity(1002).unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionUpdate {
                old: old_selection,
                new: new_selection,
            })
            .unwrap();
        let unit2_entity = selected_source.entity(1003).unwrap();
        let world = selected_source.world().unwrap();
        assert!(
            world.get::<NUnit>(unit1_entity).is_some(),
            "Old unit should still exist"
        );
        assert!(
            world.get::<NUnit>(unit2_entity).is_some(),
            "New unit should exist"
        );
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
        set_player_id_for_test(999);
        let mut selected_source = Sources::new_selected();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let unit_node = NUnit::new(1002, 0, default()).to_tnode();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node))
            .unwrap();

        let link = TNodeLink {
            id: 5004,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 0,
            solid: false,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();

        let other_player_selection = TPlayerLinkSelection {
            id: 1,
            player_id: 1000,
            parent_id: 1001,
            child_id: 1002,
            link_id: 5004,
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
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node1 = NHouseColor::new(1002, 0, default()).to_tnode();
        let color_node2 = NHouseColor::new(1003, 0, default()).to_tnode();
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
    fn test_link_processing() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();
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
        let house_node = NHouse::new(1001, 0, "house name".into()).to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node.clone()))
            .unwrap();
        let original_entity = solid_source.entity(1001).unwrap();
        let updated_node = NHouse::new(1001, 0, "house name 2".into()).to_tnode();
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
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();
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
        let ability_node = NAbilityMagic::new(1003, 0, default()).to_tnode();
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

    #[test]
    fn test_top_source_multiple_units_not_despawned() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let unit_node1 = NUnit::new(2001, 0, default()).to_tnode();
        let unit_node2 = NUnit::new(2002, 0, default()).to_tnode();
        let unit_node3 = NUnit::new(2003, 0, default()).to_tnode();

        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node1))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node2))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node3))
            .unwrap();

        let link1 = TNodeLink {
            id: 1,
            parent: 1001,
            child: 2001,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 5,
            solid: false,
        };
        let link2 = TNodeLink {
            id: 2,
            parent: 1001,
            child: 2002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 10,
            solid: false,
        };
        let link3 = TNodeLink {
            id: 3,
            parent: 1001,
            child: 2003,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 8,
            solid: false,
        };

        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link1))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link2))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link3))
            .unwrap();

        let unit1_entity = top_source.entity(2001).unwrap();
        let unit2_entity = top_source.entity(2002).unwrap();
        let unit3_entity = top_source.entity(2003).unwrap();
        assert_ne!(unit1_entity, unit2_entity, "unit 1 entity == unit 2 entity");
        assert_ne!(unit2_entity, unit3_entity, "unit 2 entity == unit 3 entity");
        assert_ne!(unit1_entity, unit3_entity, "unit 1 entity == unit 3 entity");

        let world = top_source.world_mut().unwrap();
        let units_len = world.query::<&NUnit>().iter(world).collect_vec().len();
        assert!(
            world.query::<&NUnit>().iter(world).collect_vec().len() == 3,
            "Expected 3 units to be spawned, got {units_len}"
        );
        assert!(
            world.get::<NUnit>(unit1_entity).is_some(),
            "Unit 1 should still exist in world"
        );
        assert!(
            world.get::<NUnit>(unit2_entity).is_some(),
            "Unit 2 should still exist in world"
        );
        assert!(
            world.get::<NUnit>(unit3_entity).is_some(),
            "Unit 3 should still exist in world"
        );

        let children = top_source
            .get_children_of_kind(1001, NodeKind::NUnit)
            .unwrap();
        assert_eq!(
            children.len(),
            1,
            "Should only have one linked child (top-rated)"
        );
        assert!(
            children.contains(&2002),
            "Should contain highest rated unit (2002 with rating 10)"
        );
    }

    #[test]
    fn test_solid_source_component_merging() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();

        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();

        // Component link should cause merging
        let component_link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 0,
            solid: true,
        };

        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(component_link))
            .unwrap();

        let house_entity = solid_source.entity(1001).unwrap();
        let color_entity = solid_source.entity(1002).unwrap();
        assert_eq!(
            house_entity, color_entity,
            "Component link should merge entities"
        );

        let world = solid_source.world().unwrap();
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "House should exist on merged entity"
        );
        assert!(
            world.get::<NHouseColor>(house_entity).is_some(),
            "Color should exist on merged entity"
        );
    }

    #[test]
    fn test_solid_source_owned_link_no_merging() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let unit_node = NUnit::new(1002, 0, default()).to_tnode();

        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node))
            .unwrap();

        // Owned link should NOT cause merging
        let owned_link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 0,
            solid: true,
        };

        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(owned_link))
            .unwrap();

        let house_entity = solid_source.entity(1001).unwrap();
        let unit_entity = solid_source.entity(1002).unwrap();
        assert_ne!(
            house_entity, unit_entity,
            "Owned link should NOT merge entities"
        );

        let world = solid_source.world().unwrap();
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "House should exist on original entity"
        );
        assert!(
            world.get::<NUnit>(unit_entity).is_some(),
            "Unit should exist on original entity"
        );
    }

    #[test]
    fn test_top_source_component_merging() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();

        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();

        // Component link should cause merging even in Top source
        let component_link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 5,
            solid: false,
        };
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(component_link))
            .unwrap();

        let house_entity = top_source.entity(1001).unwrap();
        let color_entity = top_source.entity(1002).unwrap();
        assert_eq!(
            house_entity, color_entity,
            "Component link should merge entities in Top source"
        );

        let world = top_source.world().unwrap();
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "House should exist on merged entity"
        );
        assert!(
            world.get::<NHouseColor>(house_entity).is_some(),
            "Color should exist on merged entity"
        );
    }

    #[test]
    fn test_chain_linked_components_merge() {
        let mut solid_source = Sources::new_solid();
        let unit_node = NUnit::new(1001, 0, "test unit".into()).to_tnode();
        let desc_node = NUnitDescription::new(
            1002,
            0,
            "description".into(),
            MagicType::Ability,
            Trigger::default(),
        )
        .to_tnode();
        let behavior_node =
            NUnitBehavior::new(1003, 0, Reaction::default(), MagicType::Ability).to_tnode();
        let repr_node = NUnitRepresentation::new(1004, 0, Material::default()).to_tnode();

        // Insert all nodes
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(desc_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(behavior_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(repr_node))
            .unwrap();

        let link = TNodeLink {
            id: 2,
            parent: 1002,
            child: 1003,
            parent_kind: "NUnitDescription".to_string(),
            child_kind: "NUnitBehavior".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();

        let link = TNodeLink {
            id: 3,
            parent: 1002,
            child: 1004,
            parent_kind: "NUnitDescription".to_string(),
            child_kind: "NUnitRepresentation".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NUnit".to_string(),
            child_kind: "NUnitDescription".to_string(),
            rating: 0,
            solid: true,
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();

        // All components should be on the same entity
        let unit_entity = solid_source.entity(1001).unwrap();
        let desc_entity = solid_source.entity(1002).unwrap();
        let behavior_entity = solid_source.entity(1003).unwrap();
        let repr_entity = solid_source.entity(1004).unwrap();

        assert_eq!(
            unit_entity, desc_entity,
            "Unit and Description should be on same entity"
        );
        assert_eq!(
            desc_entity, behavior_entity,
            "Description and Behavior should be on same entity"
        );
        assert_eq!(
            desc_entity, repr_entity,
            "Description and Representation should be on same entity"
        );

        let world = solid_source.world().unwrap();
        assert!(world.get::<NUnit>(unit_entity).is_some());
        assert!(world.get::<NUnitDescription>(unit_entity).is_some());
        assert!(world.get::<NUnitBehavior>(unit_entity).is_some());
        assert!(world.get::<NUnitRepresentation>(unit_entity).is_some());
    }

    #[test]
    fn test_top_source_component_merge_replace() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node1 = NHouseColor::new(1002, 0, default()).to_tnode();
        let color_node2 = NHouseColor::new(1003, 0, default()).to_tnode();

        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node1))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node2))
            .unwrap();

        let component_link1 = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 1,
            solid: false,
        };
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(component_link1))
            .unwrap();

        let component_link2 = TNodeLink {
            id: 2,
            parent: 1001,
            child: 1003,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 2,
            solid: false,
        };
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(component_link2))
            .unwrap();

        let house_entity = top_source.entity(1001).unwrap();
        let color_entity1 = top_source.entity(1002).unwrap();
        let color_entity2 = top_source.entity(1003).unwrap();

        let world = top_source.world_mut().unwrap();

        assert_ne!(
            house_entity, color_entity1,
            "Initial color should be moved to its own entity"
        );
        assert_eq!(
            house_entity, color_entity2,
            "Second color should get merged with house"
        );

        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "House should exist on merged entity"
        );
        assert!(
            world.get::<NHouseColor>(house_entity).is_some(),
            "Color should exist on merged entity"
        );
        assert!(
            world.get::<NHouseColor>(color_entity1).is_some(),
            "Color should exist on separate entity"
        );
    }

    #[test]
    fn test_top_source_component_remerging() {
        let mut top_source = Sources::new_top();
        let house1_node = NHouse::new(1001, 0, "house 1".into()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();
        let house2_node = NHouse::new(2001, 0, "house 2".into()).to_tnode();

        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house1_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house2_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();

        let component_link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 5,
            solid: false,
        };
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(component_link))
            .unwrap();
        let component_link = TNodeLink {
            id: 2,
            parent: 2001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 6,
            solid: false,
        };
        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(component_link))
            .unwrap();

        let house_entity1 = top_source.entity(1001).unwrap();
        let house_entity2 = top_source.entity(2001).unwrap();
        let color_entity = top_source.entity(1002).unwrap();

        let world = top_source.world_mut().unwrap();
        let houses = world.query::<&NHouse>().iter(world).len();

        assert_eq!(houses, 2, "There should be 2 houses");
        assert_eq!(
            world.get::<NHouse>(house_entity1).unwrap().name(),
            "house 1",
            "House1 name does not match"
        );
        assert_eq!(
            world.get::<NHouse>(house_entity2).unwrap().name(),
            "house 2",
            "House2 name does not match"
        );
        assert!(
            world.get::<NHouseColor>(color_entity).is_some(),
            "Color should exist on merged entity"
        );

        assert_eq!(
            house_entity2, color_entity,
            "House2 and Color should be on same entity"
        );
        assert_ne!(
            house_entity1, color_entity,
            "House1 and Color should not be on same entity"
        );
    }

    #[test]
    fn test_top_source_owned_link_no_merging() {
        let mut top_source = Sources::new_top();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let unit_node = NUnit::new(1002, 0, default()).to_tnode();

        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        top_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node))
            .unwrap();

        // Owned link should NOT cause merging in Top source
        let owned_link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NUnit".to_string(),
            rating: 5,
            solid: false,
        };

        top_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(owned_link))
            .unwrap();

        let house_entity = top_source.entity(1001).unwrap();
        let unit_entity = top_source.entity(1002).unwrap();
        assert_ne!(
            house_entity, unit_entity,
            "Owned link should NOT merge entities in Top source"
        );

        let world = top_source.world().unwrap();
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "House should exist on original entity"
        );
        assert!(
            world.get::<NUnit>(unit_entity).is_some(),
            "Unit should exist on original entity"
        );
    }

    #[test]
    fn test_selected_source_component_merging() {
        set_player_id_for_test(999);
        let mut selected_source = Sources::new_selected();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();

        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node))
            .unwrap();
        selected_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node))
            .unwrap();

        // Component link should cause merging in Selected source too
        let component_link = TNodeLink {
            id: 6001,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
            rating: 0,
            solid: true,
        };

        selected_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(component_link))
            .unwrap();

        let selection = TPlayerLinkSelection {
            id: 1,
            player_id: 999,
            parent_id: 1001,
            child_id: 1002,
            link_id: 6001,
        };
        selected_source
            .handle_stdb_update(&StdbUpdate::PlayerLinkSelectionInsert(selection))
            .unwrap();

        let house_entity = selected_source.entity(1001).unwrap();
        let color_entity = selected_source.entity(1002).unwrap();
        assert_eq!(
            house_entity, color_entity,
            "Component link should merge entities in Selected source"
        );

        let world = selected_source.world().unwrap();
        assert!(
            world.get::<NHouse>(house_entity).is_some(),
            "House should exist on merged entity"
        );
        assert!(
            world.get::<NHouseColor>(house_entity).is_some(),
            "Color should exist on merged entity"
        );
    }
}
