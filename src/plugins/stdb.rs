use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

pub fn subscribe_login(on_success: impl FnOnce() + Send + Sync + 'static) {
    let queries = ["select * from player"];
    cn().subscription_builder()
        .on_error(|e| e.event.notify_error())
        .on_applied(move |e| {
            e.event.on_success(move || {
                on_success();
            });
        })
        .subscribe(queries);
}

pub fn subscribe_game(on_success: impl FnOnce() + Send + Sync + 'static) {
    info!("Apply stdb subscriptions");
    let queries = [
        "select * from nodes_world",
        "select * from nodes_match",
        "select * from nodes_alpha",
        "select * from nodes_relations",
        "select * from battle",
    ];
    cn().subscription_builder()
        .on_error(|e| e.event.notify_error())
        .on_applied(move |e| {
            e.event.on_success(|| {
                info!("Subscription applied");
                on_success();
            });
        })
        .subscribe(queries);

    let db = cn().db();
    db.nodes_world().on_insert(|_, row| {
        let kind = NodeKind::from_str(&row.kind).unwrap();
        let id = row.id;
        info!("Node inserted {kind}");
        let data = row.data.clone();
        OperationsPlugin::add(move |world| {
            let entity = if let Some(entity) = nid_entity(id) {
                entity
            } else {
                let entity = world.spawn_empty().id();
                entity_nid_link(entity, id);
                entity
            };
            kind.unpack(entity, &data, &mut world.commands());
        });
    });
    db.nodes_world().on_update(|_, _before, row| {
        let kind = NodeKind::from_str(&row.kind).unwrap();
        let id = row.id;
        info!("Node updated {kind}");
        let data = row.data.clone();
        OperationsPlugin::add(move |world| {
            let Some(entity) = nid_entity(id) else {
                return;
            };
            match kind {
                NodeKind::Mover => {
                    let mut mover = Mover::default();
                    mover.inject_data(&data);
                    world.entity_mut(entity).insert(mover);
                }
                _ => {}
            }
        });
    });
    db.battle().on_insert(|_, row| {
        let left = Team::from_strings(0, &row.team_left).unwrap();
        let right = Team::from_strings(0, &row.team_right).unwrap();
        let battle = Battle { left, right };
        OperationsPlugin::add(move |world| {
            battle.open_window(world);
        });
    });
    db.nodes_match().on_update(|_, _, row| {
        if row.kind == NodeKind::Match.to_string() {
            let row = row.clone();
            OperationsPlugin::add(move |world| {
                MatchPlugin::load_match_data(row.id, world);
            });
        }
    });
}
