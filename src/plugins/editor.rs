use super::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorResource>();
    }
}

#[derive(Resource, Default)]
struct EditorResource {
    mode: Mode,

    team: PackedTeam,
    battle: (PackedTeam, PackedTeam),
    unit: PackedUnit,
    representation: Representation,
    vfx: Vfx,
}
fn rm(world: &mut World) -> Mut<EditorResource> {
    world.resource_mut::<EditorResource>()
}

#[derive(Default, EnumIter, AsRefStr, Clone, Copy, PartialEq, Eq, Display)]
enum Mode {
    #[default]
    Team,
    Battle,
    Unit,
    Representation,
    Vfx,
}

impl Into<String> for Mode {
    fn into(self) -> String {
        self.as_ref().to_case(Case::Title).to_string()
    }
}

impl EditorPlugin {
    fn clear(world: &mut World) {
        world.game_clear();
        TilePlugin::clear(world);
    }
    fn load_mode(world: &mut World) {
        let mode = rm(world).mode;
        info!("Load editor mode {mode}");
        Self::clear(world);
        match rm(world).mode {
            Mode::Team => {
                Tile::new(Side::Top, |ui, world| {
                    TeamContainer::new(Faction::Team)
                        .top_content(|ui, _| {
                            if Button::click("Load own").ui(ui).clicked() {
                                Confirmation::new("Open own team".cstr())
                                    .content(|ui, world| {
                                        TTeam::filter_by_owner(user_id())
                                            .filter(|t| t.pool.eq(&TeamPool::Owned))
                                            .collect_vec()
                                            .show_modified_table("Teams", ui, world, |t| {
                                                t.column_btn("select", |t, ui, world| {
                                                    rm(world).team = PackedTeam::from_id(t.id);
                                                    Confirmation::close_current(
                                                        &egui_context(world).unwrap(),
                                                    );
                                                    Self::load_mode(world);
                                                })
                                            });
                                    })
                                    .cancel(|_| {})
                                    .push(ui.ctx());
                            }
                        })
                        .ui(ui, world);
                })
                .stretch_min()
                .transparent()
                .pinned()
                .push(world);

                rm(world).team.clone().unpack(Faction::Team, world);
            }
            Mode::Battle => {}
            Mode::Unit => {}
            Mode::Representation => {}
            Mode::Vfx => {}
        }
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            if EnumSwitcher::show(&mut rm(world).mode, ui) {
                Self::load_mode(world);
            }
        })
        .pinned()
        .no_expand()
        .transparent()
        .keep()
        .push(world);
        Self::load_mode(world);
    }
}
