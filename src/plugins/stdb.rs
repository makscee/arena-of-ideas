use spacetimedb_sdk::DbContext;

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
        info!("Node insert {kind}");
        let data = row.data.clone();
        OperationsPlugin::add(move |world| {
            kind.unpack(world.spawn_empty().id(), &data, &mut world.commands());
        });
    });
}
