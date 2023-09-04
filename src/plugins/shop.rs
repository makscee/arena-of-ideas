use super::*;

use rand::seq::SliceRandom;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveTeam>()
            .add_systems(OnEnter(GameState::Shop), Self::enter_state)
            .add_systems(PostUpdate, Self::input)
            .add_systems(Update, Self::ui);
    }
}

impl ShopPlugin {
    fn enter_state(world: &mut World) {
        if let Some(team) = &world.resource::<ActiveTeam>().team {
            team.clone().unpack(Faction::Team, world);
        }
        Self::fill_showcase(world);
    }

    fn leave_state(world: &mut World) {}

    fn input(world: &mut World) {
        if just_pressed(KeyCode::P, world) {
            Self::pack_active_team(world);
            UnitPlugin::despawn_all(world);
            Self::unpack_active_team(world);
        }
    }

    fn fill_showcase(world: &mut World) {
        let mut units = Vec::default();
        let pool = Pools::heroes(world).into_values().collect_vec();
        for _ in 0..5 {
            let unit = (*pool.choose(&mut rand::thread_rng()).unwrap()).clone();
            units.push(unit);
        }
        for unit in units {
            unit.unpack(Faction::Shop, None, world);
        }
        UnitPlugin::fill_slot_gaps(Faction::Shop, world);
        UnitPlugin::translate_to_slots(world);
    }

    pub fn pack_active_team(world: &mut World) {
        let team = PackedTeam::pack(Faction::Team, world);
        let mut active_team = world.get_resource_mut::<ActiveTeam>().unwrap();
        active_team.team = Some(team);
    }

    pub fn unpack_active_team(world: &mut World) {
        world
            .get_resource::<ActiveTeam>()
            .unwrap()
            .team
            .clone()
            .expect("Tried to unpack emtpy Active Team")
            .unpack(Faction::Team, world);
        UnitPlugin::translate_to_slots(world);
    }

    pub fn ui(world: &mut World) {
        let context = egui_context(world);
        for unit in UnitPlugin::collect_faction(Faction::Shop, world) {
            let window = UnitPlugin::draw_unit_panel(unit, vec2(0.0, -1.5), world);
            window.show(&context, |ui| {
                ui.vertical_centered(|ui| {
                    let btn = ui.button("Buy");
                    if btn.clicked() {
                        VarState::push_back(
                            unit,
                            VarName::Faction,
                            Change::new(VarValue::Faction(Faction::Team)),
                            world,
                        );
                        VarState::push_back(
                            unit,
                            VarName::Slot,
                            Change::new(VarValue::Int(0)),
                            world,
                        );
                        UnitPlugin::fill_slot_gaps(Faction::Team, world);
                        UnitPlugin::translate_to_slots(world);
                    }
                })
            });
        }
    }
}

#[derive(Resource, Default)]
pub struct ActiveTeam {
    pub team: Option<PackedTeam>,
}
