use spacetimedb_sdk::DbContext;

use super::*;

pub fn subscribe_game(on_success: impl FnOnce() + Send + Sync + 'static) {
    info!("Apply stdb subscriptions");
    cn().subscription_builder()
        .on_error(|e| e.event.notify_error())
        .on_applied(move |e| {
            e.event.on_success(|| {
                info!("Subscription applied");
                on_success();
                subscribe_table_updates();
                OperationsPlugin::add(|world| {
                    All::load_recursive(0)
                        .unwrap()
                        .unpack(world.spawn_empty().id(), world);
                });
            });
        })
        .subscribe(["select * from tnodes", "select * from nodes_relations"]);
}
fn subscribe_table_updates() {
    let db = cn().db();
    db.tnodes().on_insert(|_, row| {
        let kind = NodeKind::from_str(&row.kind).unwrap();
        let id = row.id;
        info!("Node inserted {kind}");
        let data = row.data.clone();
        OperationsPlugin::add(move |world| {
            let entity = if let Some(entity) = world.get_id_link(id) {
                entity
            } else {
                let entity = world.spawn_empty().id();
                world.add_id_link(id, entity);
                entity
            };
            kind.unpack(entity, &data, Some(id), world);
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
}

pub fn subscribe_reducers() {
    cn().reducers.on_match_insert(|e| {
        e.event.notify_error();
    });
    cn().reducers.on_match_edit_fusions(|e, _| {
        if !e.check_identity() {
            return;
        }
        e.event.notify_error();
        todo!();
    });
}
