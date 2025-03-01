use spacetimedb_sdk::DbContext;

use super::*;

pub struct StdbPlugin;

impl Plugin for StdbPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StdbData>()
            .add_systems(Update, Self::update);
    }
}

#[derive(Resource, Default)]
struct StdbData {
    nodes_queue: Vec<TNode>,
}

impl StdbPlugin {
    fn update(world: &mut World) {
        if !world.is_resource_changed::<StdbData>() {
            let q = world.resource::<StdbData>().nodes_queue.len();
            if q > 0 {
                info!("Nodes in queue: {q}");
            }
            return;
        }
        world.resource_scope(|world, mut d: Mut<StdbData>| {
            d.nodes_queue.retain(|node| {
                if let Some(entity) = world.get_id_link(node.id) {
                    node.unpack(entity, world);
                    return false;
                }
                let Some(rel) = cn().db.nodes_relations().id().find(&node.id) else {
                    return true;
                };
                let Some(parent) = world.get_id_link(rel.parent) else {
                    return true;
                };
                node.unpack(world.spawn_empty().set_parent(parent).id(), world);
                false
            });
        });
    }
}

static CORE_UNIT_NAME_LINKS: OnceCell<Mutex<HashMap<String, Entity>>> = OnceCell::new();
pub fn core_unit_by_name(name: &str) -> Result<Entity, ExpressionError> {
    CORE_UNIT_NAME_LINKS
        .get()
        .unwrap()
        .lock()
        .get(name)
        .copied()
        .to_e_fn(|| format!("Core unit {name} not found"))
}

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
                    let pid = player_id();
                    let entity = world
                        .get_id_link(pid)
                        .expect(&format!("Player#{pid} not found"));
                    save_player_entity(entity);
                    let units: HashMap<String, Entity> = HashMap::from_iter(
                        All::get_by_id(0, world)
                            .unwrap()
                            .core_load(world)
                            .unwrap()
                            .into_iter()
                            .filter_map(|h| h.units_load(world).ok())
                            .flatten()
                            .map(|u| (u.name.clone(), u.entity())),
                    );
                    *CORE_UNIT_NAME_LINKS
                        .get_or_init(|| HashMap::default().into())
                        .lock() = units;
                });
            });
        })
        .subscribe(["select * from tnodes", "select * from nodes_relations"]);
}
fn subscribe_table_updates() {
    let db = cn().db();
    db.tnodes().on_insert(|_, node| {
        info!("Node inserted {}#{}", node.kind, node.id);
        let node = node.clone();
        OperationsPlugin::add(move |world| {
            world.resource_mut::<StdbData>().nodes_queue.push(node);
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
