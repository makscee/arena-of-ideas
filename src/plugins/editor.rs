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

#[derive(Default, EnumIter, AsRefStr, Clone, Copy, PartialEq, Eq)]
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
    }
    fn load_mode(world: &mut World) {
        match rm(world).mode {
            Mode::Team => {
                Tile::new(Side::Top, |ui, world| {
                    TeamContainer::new(Faction::Team)
                        .top_content(|ui, world| {
                            if Button::click("Load own").ui(ui).clicked() {
                                Confirmation::new("Open own team".cstr())
                                    .content(|ui, world| {})
                                    .push(ui.ctx());
                            }
                        })
                        .ui(ui, world);
                })
                .stretch_min()
                .transparent()
                .pinned()
                .push(world);
            }
            Mode::Battle => {}
            Mode::Unit => {}
            Mode::Representation => {}
            Mode::Vfx => {}
        }
    }
    pub fn add_tiles(world: &mut World) {
        Self::load_mode(world);
        Tile::new(Side::Top, |ui, world| {
            if EnumSwitcher::show(&mut rm(world).mode, ui) {
                Self::load_mode(world);
            }
        })
        .pinned()
        .no_expand()
        .transparent()
        .push(world);
    }
}
