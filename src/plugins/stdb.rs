use spacetimedb_sdk::{DbContext, TableWithPrimaryKey};

use super::*;

fn on_subscription_applied() {
    OperationsPlugin::add(|world| {
        let mut unit = Unit::from_table(
            NodeDomain::Alpha,
            NodeDomain::Alpha.filter_by_kind(NodeKind::Unit)[0].id,
        )
        .unwrap();
        for (var, value) in unit.get_all_vars() {
            dbg!(var, value);
        }
        Window::new("Unit df", move |ui, _| {
            ui.columns(2, |ui| {
                unit.show(None, &Context::default().set_owner_node(&unit), &mut ui[0]);
                unit.show_mut(None, &mut ui[1]);
            });
        })
        .push(world);
    });
}
pub fn db_subscriptions() {
    info!("Apply stdb subscriptions");
    let queries = [
        "select * from nodes_world",
        "select * from nodes_match",
        "select * from nodes_alpha",
        "select * from battle",
        "select * from nodes_relations",
    ];
    cn().subscription_builder()
        .on_error(|e| e.event.notify_error())
        .on_applied(move |e| {
            e.event.on_success(|_| {
                info!("Subscription applied");
            });
            on_subscription_applied();
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
        dbg!(row);
        let left = Team::from_strings(0, &row.team_left).unwrap();
        let right = Team::from_strings(0, &row.team_right).unwrap();
        let battle = Battle { left, right };
        OperationsPlugin::add(move |world| {
            battle.open_window(world);
        });
    });

    cn().reducers.on_sync_assets(|e, _, _| {
        e.event.notify_error();
        info!("{}", "Assets sync done".blue());
        app_exit_op();
    });
}
