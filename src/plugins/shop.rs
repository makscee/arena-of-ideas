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
        unit.clone()
            .unpack(TeamPlugin::entity(Faction::Shop, world), None, world);
        UnitPlugin::fill_slot_gaps(Faction::Shop, world);
    }
    fn exit(world: &mut World) {}
    pub fn show_container(ui: &mut Ui, world: &mut World) {
        TopMenu::new(vec!["Container Config"]).ui(ui);
        let data = world.resource::<ShopData>();
        TeamContainer::new(
            ui.ctx().screen_rect().center() + egui::vec2(0.0, -data.case_height),
            Faction::Shop,
            egui::vec2(data.unit_offset, 0.0),
        )
        .ui(ui, world);
        Tile::left("Container Config")
            .content(move |ui, world| {
                let mut data = world.resource_mut::<ShopData>();
                Slider::new("offset")
                    .range(10.0..=400.0)
                    .ui(&mut data.unit_offset, ui);
            })
            .ui(ui, world);
    }
}
