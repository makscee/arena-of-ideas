use super::*;

pub struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MetaResource>();
    }
}

#[derive(Resource, Default)]
struct MetaResource {
    state: SubState,
}

#[derive(PartialEq, Copy, Clone, EnumIter, Display, Default)]
enum SubState {
    #[default]
    Inventory,
    Shop,
}

impl MetaPlugin {
    pub fn ui_tiles(ctx: &egui::Context, world: &mut World) {
        let mut r = world.resource_mut::<MetaResource>();
        let state = SubsectionMenu::new(r.state).show(ctx);
        r.state = state;
        Tile::left("Meta").open().show(ctx, |ui| match state {
            SubState::Inventory => {
                TItem::iter()
                    .collect_vec()
                    .show_table("Inventory", ui, world);
            }
            SubState::Shop => {
                TMetaShop::iter()
                    .collect_vec()
                    .show_table("Meta Shop", ui, world);
            }
        });
    }
}
