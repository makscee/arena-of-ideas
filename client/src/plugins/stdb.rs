use crate::login;

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
                    if sources
                        .iter_mut()
                        .any(|s| s.handle_stdb_update(&update).is_err())
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
    db.votes().on_update(|_, old, new| {
        if old.player_id == player_id() {
            if old.upvotes != new.upvotes {
                format!(
                    "[green ⬆️] Upvotes {}",
                    (new.upvotes - old.upvotes).cstr_expanded()
                )
                .notify_op();
            } else if old.downvotes != new.downvotes {
                format!(
                    "[red ⬇️] Downvotes {}",
                    (new.downvotes - old.downvotes).cstr_expanded()
                )
                .notify_op();
            }
        }
    });
}

fn queue_update(update: StdbUpdate) {
    UPDATE_QUEUE.lock().push_back(update);
    set_need_update(true);
}

fn default_callback(e: &ReducerEventContext) {
    if !e.check_identity() {
        return;
    }
    e.event.notify_error();
}

pub fn subscribe_reducers() {
    for r in reducer_registry::AllReducers::iter() {
        subscribe_reducer(r);
    }
}

fn subscribe_reducer(reducer: reducer_registry::AllReducers) {
    use reducer_registry::AllReducers;

    let r = &cn().reducers;
    match reducer {
        AllReducers::AdminAddGold => {
            r.on_admin_add_gold(|e| default_callback(e));
        }
        AllReducers::AdminAddVotes => {
            r.on_admin_add_votes(|e, _| default_callback(e));
        }
        AllReducers::AdminDailyUpdate => {
            r.on_admin_daily_update(|e| default_callback(e));
        }
        AllReducers::AdminDeleteNodeRecursive => {
            r.on_admin_delete_node_recursive(|e, _| default_callback(e));
        }
        AllReducers::AdminUploadWorld => {
            r.on_admin_upload_world(|e, _, _, _| default_callback(e));
        }
        AllReducers::ContentCheckPhaseCompletion => {
            r.on_content_check_phase_completion(|e| default_callback(e));
        }
        AllReducers::ContentDeleteNode => {
            r.on_content_delete_node(|e, _| default_callback(e));
        }
        AllReducers::ContentDownvoteNode => {
            r.on_content_downvote_node(|e, _| default_callback(e));
        }
        AllReducers::ContentPublishNode => {
            r.on_content_publish_node(|e, _, _| default_callback(e));
        }
        AllReducers::ContentResetCore => {
            r.on_content_reset_core(|e| default_callback(e));
        }
        AllReducers::ContentSuggestNode => {
            r.on_content_suggest_node(|e, _, _| default_callback(e));
        }
        AllReducers::ContentUpvoteNode => {
            r.on_content_upvote_node(|e, _| default_callback(e));
        }
        AllReducers::DailyUpdateReducer => {
            r.on_daily_update_reducer(|e, _| default_callback(e));
        }
        AllReducers::IdentityDisconnected => {
            r.on_identity_disconnected(|e| default_callback(e));
        }
        AllReducers::Login => {
            r.on_login(|e, _, _| default_callback(e));
        }
        AllReducers::LoginByIdentity => {
            r.on_login_by_identity(|e| default_callback(e));
        }
        AllReducers::Logout => {
            r.on_logout(|e| default_callback(e));
        }
        AllReducers::MatchAbandon => {
            r.on_match_abandon(|e| default_callback(e));
        }
        AllReducers::MatchBenchUnit => {
            r.on_match_bench_unit(|e, _| default_callback(e));
        }
        AllReducers::MatchBossBattle => {
            r.on_match_boss_battle(|e| default_callback(e));
        }
        AllReducers::MatchCancelFusion => {
            r.on_match_cancel_fusion(|e| default_callback(e));
        }
        AllReducers::MatchChooseFusion => {
            r.on_match_choose_fusion(|e, _| default_callback(e));
        }
        AllReducers::MatchComplete => {
            r.on_match_complete(|e| default_callback(e));
        }
        AllReducers::MatchInsert => {
            r.on_match_insert(|e| default_callback(e));
        }
        AllReducers::MatchMoveUnit => {
            r.on_match_move_unit(|e, _, _| default_callback(e));
        }
        AllReducers::MatchSellUnit => {
            r.on_match_sell_unit(|e, _| default_callback(e));
        }
        AllReducers::MatchShopBuy => {
            r.on_match_shop_buy(|e, _| default_callback(e));
        }
        AllReducers::MatchShopReroll => {
            r.on_match_shop_reroll(|e| default_callback(e));
        }
        AllReducers::MatchStackUnit => {
            r.on_match_stack_unit(|e, _, _| default_callback(e));
        }
        AllReducers::MatchStartBattle => {
            r.on_match_start_battle(|e| default_callback(e));
        }
        AllReducers::MatchStartFusion => {
            r.on_match_start_fusion(|e, _, _| default_callback(e));
        }
        AllReducers::Register => {
            r.on_register(|e, _, _| default_callback(e));
        }
        AllReducers::SetPassword => {
            r.on_set_password(|e, _, _| default_callback(e));
        }
        AllReducers::MatchSubmitBattleResult => {
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
        }
    }
}

mod reducer_registry {
    include!(concat!(env!("OUT_DIR"), "/generated_reducers.rs"));
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
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node.clone()))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node.clone()))
            .unwrap();
        link(&mut solid_source, &house_node, &color_node);
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
    fn test_link_processing() {
        let mut solid_source = Sources::new_solid();
        let house_node = NHouse::new(1001, 0, default()).to_tnode();
        let color_node = NHouseColor::new(1002, 0, default()).to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node.clone()))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node.clone()))
            .unwrap();
        static LINK_ID_TEMP: AtomicU64 = AtomicU64::new(100);
        let id = LINK_ID_TEMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let link = TNodeLink {
            id,
            parent: house_node.id,
            child: color_node.id,
            parent_kind: house_node.kind.clone(),
            child_kind: color_node.kind.clone(),
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
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node.clone()))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node.clone()))
            .unwrap();
        let house_entity = solid_source.entity(1001).unwrap();
        let color_entity = solid_source.entity(1002).unwrap();
        assert_ne!(
            house_entity, color_entity,
            "Should start on different entities"
        );
        link(&mut solid_source, &house_node, &color_node);
        let ability_node = NAbilityMagic::new(1003, 0, default()).to_tnode();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(ability_node.clone()))
            .unwrap();
        link(&mut solid_source, &house_node, &ability_node);
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
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node.clone()))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(color_node.clone()))
            .unwrap();

        link(&mut solid_source, &house_node, &color_node);

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
            .handle_stdb_update(&StdbUpdate::NodeInsert(house_node.clone()))
            .unwrap();
        solid_source
            .handle_stdb_update(&StdbUpdate::NodeInsert(unit_node.clone()))
            .unwrap();

        link(&mut solid_source, &house_node, &unit_node);

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
}
