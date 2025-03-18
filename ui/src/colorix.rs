use egui_colors::Colorix;

use super::*;

#[derive(Resource, Default)]
pub struct ColorixData {
    pub colorix: Colorix,
}

pub fn setup_colorix(world: &mut World) {
    let ctx = &egui_context(world).unwrap();
    let colorix = Colorix::global(ctx, egui_colors::utils::COOL);
    world.insert_resource(ColorixData { colorix });
}

pub fn colorix_editor(ui: &mut Ui, world: &mut World) {
    let colorix = &mut world.colorix_mut().colorix;
    colorix.custom_picker(ui);
    colorix.twelve_from_custom(ui);
}

pub trait ColorixExt {
    fn colorix(&self) -> &Colorix;
    fn colorix_mut(&mut self) -> Mut<ColorixData>;
}

impl ColorixExt for World {
    fn colorix(&self) -> &Colorix {
        &self.resource::<ColorixData>().colorix
    }
    fn colorix_mut(&mut self) -> Mut<ColorixData> {
        self.resource_mut::<ColorixData>()
    }
}
