use super::*;

static UPDATE_QUEUE: once_cell::sync::Lazy<Mutex<VecDeque<StdbUpdate>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(VecDeque::new()));
static NEED_UPDATE: Mutex<bool> = Mutex::new(false);

fn set_need_update(value: bool) {
    *NEED_UPDATE.lock() = value;
}

fn is_need_update() -> bool {
    *NEED_UPDATE.lock()
}

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::process_update_queue);
    }
}

impl StdbPlugin {
    fn process_update_queue() {
        if !is_need_update() {
            return;
        }
        set_need_update(false);
        let mut queue = UPDATE_QUEUE.lock();
        if queue.is_empty() {
            return;
        }
        with_static_sources(|sources| {
            loop {
                let mut changed = false;
                for _ in 0..queue.len() {
                    let update = queue.pop_front().unwrap();
                    if sources.solid.handle_stdb_update(&update).is_err()
                        || sources.core.handle_stdb_update(&update).is_err()
                    {
                        queue.push_back(update);
                        continue;
                    }
                    changed = true;
                }
                if !changed {
                    debug!("db events queue left: {}", queue.len());
                    // dbg!(&queue);
                    break;
                }
            }
        });
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
            op(|world| {
                world.init_resource::<TablesSubscribeOption>();
            });
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

        // Check if this is the current player's NMatch and if battle state changed
        if new.kind() == NodeKind::NMatch && new.owner == player_id() {
            // Parse the data to check state field
            if let (Ok(old_match), Ok(new_match)) =
                (old.to_node::<NMatch>(), new.to_node::<NMatch>())
            {
                if !old_match.state.is_battle() && new_match.state.is_battle() {
                    // Battle state became active, trigger state transition
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
}

fn queue_update(update: StdbUpdate) {
    UPDATE_QUEUE.lock().push_back(update);
    set_need_update(true);
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
    cn().reducers.on_match_boss_battle(|e| {
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

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
    fn test_node_create_link_delete() {
        let mut solid_source = Sources::new_solid();
        let player_node = NPlayer::new(1001, 1, "test player".into()).to_tnode();
        let match_node = NMatch::new(
            1002,
            1,
            10,
            1,
            1,
            true,
            default(),
            default(),
            default(),
            default(),
        )
        .to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(player_node))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(match_node.clone()))
            .unwrap();
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NPlayer".into(),
            child_kind: "NMatch".into(),
        };
        solid_source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link.clone()))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeDelete(match_node))
            .unwrap();

        let entity = solid_source.entity(1001).expect("Node entity should exist");
        let world = solid_source.world().expect("World should be accessible");
        assert!(
            world.get::<NPlayer>(entity).is_some(),
            "NPlayer component should exist"
        );
        let player = world.get::<NPlayer>(entity).unwrap();
        assert_eq!(player.id, 1001);
        assert_eq!(player.player_name, "test player");
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
        let mut top_source = Sources::new_core();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();
        let link = TNodeLink {
            id: 1,
            parent: 1001,
            child: 1002,
            parent_kind: "NHouse".to_string(),
            child_kind: "NHouseColor".to_string(),
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
            "House and Color should be on same entities in Core source"
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

    fn link(source: &mut Sources, parent: &TNode, child: &TNode) {
        static LINK_ID: AtomicU64 = AtomicU64::new(1);
        let id = LINK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let link = TNodeLink {
            id,
            parent: parent.id,
            child: child.id,
            parent_kind: parent.kind.clone(),
            child_kind: child.kind.clone(),
        };
        source
            .handle_stdb_update(&StdbUpdate::LinkInsert(link))
            .unwrap();
    }

    #[test]
    fn test_many_to_one_cloning() {
        let source = &mut Sources::new_solid();
        let unit1 = NUnit::new(1001, 0, "test1".into()).to_tnode();
        let unit2 = NUnit::new(1002, 0, "test2".into()).to_tnode();
        let unit3 = NUnit::new(1003, 0, "test3".into()).to_tnode();
        let unit4 = NUnit::new(1004, 0, "test4".into()).to_tnode();
        let description = NUnitDescription::default().with_id(2000).to_tnode();
        let stats1 = NUnitStats::new(3001, 0, 1, 1).to_tnode();
        let stats2 = NUnitStats::new(3002, 0, 2, 2).to_tnode();
        source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit1.clone()))
            .unwrap();
        source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit2.clone()))
            .unwrap();
        source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit3.clone()))
            .unwrap();
        source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit4.clone()))
            .unwrap();
        source
            .handle_stdb_update(&StdbUpdate::NodeInsert(description.clone()))
            .unwrap();
        source
            .handle_stdb_update(&StdbUpdate::NodeInsert(stats1.clone()))
            .unwrap();
        source
            .handle_stdb_update(&StdbUpdate::NodeInsert(stats2.clone()))
            .unwrap();
        link(source, &unit1, &description);
        link(source, &unit2, &description);
        link(source, &unit3, &description);
        link(source, &unit4, &description);
        link(source, &unit1, &stats1);
        link(source, &unit2, &stats1);
        link(source, &unit3, &stats2);
        link(source, &unit4, &stats2);
        source.exec_context(|ctx| {
            assert_eq!(
                ctx.get_var_inherited(unit1.id, VarName::hp).unwrap(),
                VarValue::i32(1)
            );
            assert_eq!(
                ctx.get_var_inherited(unit1.id, VarName::pwr).unwrap(),
                VarValue::i32(1)
            );
            assert_eq!(
                ctx.get_var_inherited(unit2.id, VarName::hp).unwrap(),
                VarValue::i32(1)
            );
            assert_eq!(
                ctx.get_var_inherited(unit2.id, VarName::pwr).unwrap(),
                VarValue::i32(1)
            );
            assert_eq!(
                ctx.get_var_inherited(unit3.id, VarName::hp).unwrap(),
                VarValue::i32(2)
            );
            assert_eq!(
                ctx.get_var_inherited(unit3.id, VarName::pwr).unwrap(),
                VarValue::i32(2)
            );
            assert_eq!(
                ctx.get_var_inherited(unit4.id, VarName::hp).unwrap(),
                VarValue::i32(2)
            );
            assert_eq!(
                ctx.get_var_inherited(unit4.id, VarName::pwr).unwrap(),
                VarValue::i32(2)
            );
        });
    }

    #[test]
    fn test_chain_linked_components_merge() {
        let mut solid_source = Sources::new_solid();
        let unit_node = NUnit::new(1001, 0, "test unit".into()).to_tnode();
        let desc_node = NUnitDescription::new(1002, 0, "description".into()).to_tnode();
        let behavior_node = NUnitBehavior::new(1003, 0, Reaction::default()).to_tnode();
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
}
