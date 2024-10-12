use super::*;

pub struct UnitEditorPlugin;

impl Plugin for UnitEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UnitEditor), Self::on_enter)
            .init_resource::<UnitEditorResource>();
    }
}

#[derive(Resource, Default)]
struct UnitEditorResource {
    team: PackedTeam,
}
fn rm(world: &mut World) -> Mut<UnitEditorResource> {
    world.resource_mut::<UnitEditorResource>()
}

impl UnitEditorPlugin {
    fn on_enter(world: &mut World) {
        rm(world).team.clone().unpack(Faction::Left, world);
    }
    pub fn load_team(team: PackedTeam, world: &mut World) {
        rm(world).team = team;
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            TeamContainer::new(Faction::Left)
                .right_to_left()
                .ui(ui, world);
        })
        .pinned()
        .transparent()
        .min_space(egui::vec2(300.0, 0.0))
        .no_expand()
        .push(world);
        Tile::new(Side::Right, |ui, world| {
            TeamContainer::new(Faction::Right).ui(ui, world);
        })
        .pinned()
        .transparent()
        .min_space(egui::vec2(300.0, 0.0))
        .no_expand()
        .push(world);
    }
}
