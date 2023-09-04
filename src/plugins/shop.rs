use super::*;
use bevy_egui::{egui::Pos2, *};

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
        } else {
            let mut units = Vec::default();
            for (_, unit) in Pools::heroes(world).into_iter() {
                units.push(unit.clone());
            }
            for unit in units {
                unit.unpack(Faction::Team, None, world);
            }
            UnitPlugin::fill_slot_gaps(Faction::Team, world);
        }
        UnitPlugin::translate_to_slots(world);
    }

    fn leave_state(world: &mut World) {}

    fn input(world: &mut World) {
        if just_pressed(KeyCode::P, world) {
            Self::pack_active_team(world);
            UnitPlugin::despawn_all(world);
            Self::unpack_active_team(world);
        }
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

    pub fn ui(mut contexts: EguiContexts) {
        // egui::Window::new("Shop")
        //     .fixed_rect(egui::Rect::from_center_size(
        //         Pos2::new(300.0, 10.0),
        //         egui::vec2(200.0, 50.0),
        //     ))
        //     .show(contexts.ctx_mut(), |ui| {
        //         ui.button("Click").clicked();

        //     });
    }
}

#[derive(Resource, Default)]
pub struct ActiveTeam {
    pub team: Option<PackedTeam>,
}
