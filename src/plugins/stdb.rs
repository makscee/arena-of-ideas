use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

pub fn db_subscriptions() {
    info!("Apply stdb subscriptions");
    let queries = ["select * from nodes"];
    cn().subscription_builder()
        .on_error(|e| e.event.notify_error())
        .on_applied(move |e| {
            e.event.on_success(|_| {
                info!("Subscription applied");
            });
        })
        .subscribe(queries);

    let db = cn().db();
    db.nodes().on_insert(|_, row| {
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
    db.nodes().on_update(|_, _before, row| {
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

    cn().reducers.on_sync_assets(|e, _| {
        e.event.notify_error();
        info!("{}", "Assets sync done".blue());
        app_exit_op();
    });
}
