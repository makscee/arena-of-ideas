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
}

impl Default for ShopData {
    fn default() -> Self {
        Self { case_height: 20.0 }
    }
}

impl ShopPlugin {
    fn enter(world: &mut World) {
        let unit = GameAssets::get(world)
            .heroes
            .values()
            .next()
            .unwrap()
            .clone();
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Shop, world), None, world);
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Shop, world), None, world);
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Shop, world), None, world);
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Shop, world), None, world);
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Shop, world), None, world);
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Team, world), None, world);
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Team, world), None, world);
        UnitPlugin::fill_slot_gaps(Faction::Shop, world);
        UnitPlugin::fill_slot_gaps(Faction::Team, world);
    }
    fn exit(world: &mut World) {
        world.game_clear();
    }
    pub fn show_containers(ui: &mut Ui, world: &mut World) {
        let mut wd = world.remove_resource::<WidgetData>().unwrap();
        let sd = world.resource::<ShopData>().clone();

        UnitContainer::new(Faction::Shop)
            .direction(Side::Top)
            .offset([0.0, -sd.case_height])
            .slots(6)
            .slot_content(|_, ui, _| {
                ui.vertical_centered_justified(|ui| {
                    Button::click("-3 G").title("buy").ui(ui);
                });
            })
            .ui(&mut wd, ui, world);
        UnitContainer::new(Faction::Team)
            .direction(Side::Bottom)
            .offset([0.0, sd.case_height])
            .slots(5)
            .ui(&mut wd, ui, world);
        world.insert_resource(wd);
    }
}
