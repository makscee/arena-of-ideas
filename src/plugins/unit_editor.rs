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
    left: PackedTeam,
    right: PackedTeam,
}
fn rm(world: &mut World) -> Mut<UnitEditorResource> {
    world.resource_mut::<UnitEditorResource>()
}

impl UnitEditorPlugin {
    fn on_enter(world: &mut World) {
        Self::spawn_teams(world);
    }
    fn spawn_teams(world: &mut World) {
        TeamPlugin::despawn(Faction::Left, world);
        TeamPlugin::despawn(Faction::Right, world);
        rm(world).left.clone().unpack(Faction::Left, world);
        rm(world).right.clone().unpack(Faction::Right, world);
    }
    pub fn load_team_left(team: PackedTeam, world: &mut World) {
        rm(world).left = team;
    }
    pub fn load_team_right(team: PackedTeam, world: &mut World) {
        rm(world).right = team;
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                if Button::click("Load sample".into()).ui(ui).clicked() {
                    Self::load_team_left(
                        PackedTeam {
                            units: [ron::from_str("(hp: 10, pwr: 1)").unwrap()].into(),
                            ..default()
                        },
                        world,
                    );
                    Self::load_team_right(
                        PackedTeam {
                            units: [ron::from_str("(hp: 10, pwr: 1)").unwrap()].into(),
                            ..default()
                        },
                        world,
                    );
                    Self::spawn_teams(world);
                }
                if Button::click("Strike".into()).ui(ui).clicked() {
                    if let Some((left, right)) = BattlePlugin::get_strikers(world) {
                        BattlePlugin::run_strike(left, right, world);
                    }
                }
            });
        })
        .pinned()
        .transparent()
        .non_focusable()
        .push(world);
        Tile::new(Side::Top, |ui, world| {
            if ui.available_width() < 10.0 {
                return;
            }
            ui.add_space(100.0);
            ui.columns(2, |ui| {
                TeamContainer::new(Faction::Left)
                    .right_to_left()
                    .ui(&mut ui[0], world);
                TeamContainer::new(Faction::Right).ui(&mut ui[1], world);
            });
        })
        .max()
        .transparent()
        .pinned()
        .push(world);
    }
}
