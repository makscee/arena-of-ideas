use spacetimedb_sdk::{
    once_on_subscription_applied,
    table::{TableWithPrimaryKey, UpdateCallbackId},
};

use super::*;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::enter)
            .add_systems(OnExit(GameState::Shop), Self::exit)
            .init_resource::<ShopData>();
    }
}

#[derive(Resource, Clone)]
pub struct ShopData {
    pub case_height: f32,
    callback: Option<UpdateCallbackId<Run>>,
}

impl Default for ShopData {
    fn default() -> Self {
        Self {
            case_height: 0.0,
            callback: None,
        }
    }
}

impl ShopPlugin {
    fn enter(mut sd: ResMut<ShopData>) {
        run_start();
        ServerPlugin::subscribe_run();
        once_on_subscription_applied(|| {
            OperationsPlugin::add(|world| {
                Self::sync_run(Run::current(), world);
            });
        });
        let cb = Run::on_update(|_, run, _| {
            let run = run.clone();
            OperationsPlugin::add(|world| Self::sync_run(run, world))
        });
        sd.callback = Some(cb);
    }
    fn exit(world: &mut World) {
        world.game_clear();
        if let Some(cb) = world.resource_mut::<ShopData>().callback.take() {
            Run::remove_on_update(cb);
        }
    }
    fn sync_run(run: Run, world: &mut World) {
        debug!("Sync run");
        let mut shop_units: HashMap<u64, Entity> = HashMap::from_iter(
            UnitPlugin::collect_faction(Faction::Shop, world)
                .into_iter()
                .map(|e| (VarState::get(e, world).id(), e)),
        );
        let team = TeamPlugin::entity(Faction::Shop, world);
        for (
            i,
            ShopSlot {
                unit,
                available,
                id,
                ..
            },
        ) in run.shop.into_iter().enumerate()
        {
            let slot = i as i32 + 1;
            if available {
                if shop_units.contains_key(&id) {
                    shop_units.remove(&id);
                    continue;
                }
                GameAssets::get(world)
                    .heroes
                    .get(&unit)
                    .cloned()
                    .unwrap()
                    .unpack(team, Some(slot), Some(id), world);
            }
        }
        for entity in shop_units.values() {
            UnitPlugin::despawn(*entity, world);
        }
    }
    pub fn show_containers(wd: &mut WidgetData, ui: &mut Ui, world: &mut World) {
        let sd = world.resource::<ShopData>().clone();
        if let Some(run) = Run::get_current() {
            let shop = run.shop;
            UnitContainer::new(Faction::Shop)
                .direction(Side::Top)
                .offset([0.0, -sd.case_height])
                .slots(shop.len())
                .slot_content(move |slot, ui, _| {
                    ui.vertical_centered_justified(|ui| {
                        let ind = slot - 1;
                        let ss = &shop[ind];
                        if ss.available {
                            if Button::click(format!("-{} G", ss.price))
                                .title("buy".into())
                                .ui(ui)
                                .clicked()
                            {
                                shop_buy(ind as u8);
                            }
                        }
                    });
                })
                .ui(wd, ui, world);
            UnitContainer::new(Faction::Team)
                .direction(Side::Bottom)
                .offset([0.0, sd.case_height])
                .slots(5)
                .ui(wd, ui, world);
        }
    }
}
