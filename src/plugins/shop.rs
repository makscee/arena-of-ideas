use super::*;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shop), Self::enter)
            .add_systems(OnExit(GameState::Shop), Self::exit)
            .init_resource::<ShopData>();
    }
}

#[derive(Resource)]
struct ShopData {
    case_height: f32,
    unit_offset: f32,
}

impl Default for ShopData {
    fn default() -> Self {
        Self {
            case_height: 30.0,
            unit_offset: 130.0,
        }
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
        UnitPlugin::fill_slot_gaps(Faction::Shop, world);
    }
    fn exit(world: &mut World) {
        world.game_clear();
    }
    pub fn show_container(ui: &mut Ui, world: &mut World) {
        TopMenu::new(vec!["Container Config"]).ui(ui);
        let mut data = world.remove_resource::<WidgetData>().unwrap();

        UnitContainer::new(Faction::Shop)
            .slots(6)
            .ui(&mut data, ui);
        Tile::left("Container Config")
            .content(move |ui, world| {
                let mut data = world.resource_mut::<ShopData>();
                Slider::new("offset")
                    .range(10.0..=400.0)
                    .ui(&mut data.unit_offset, ui);
            })
            .ui(ui, world);
        world.insert_resource(data);
    }
}
