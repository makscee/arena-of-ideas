use super::*;

pub struct GameAssetsEditor;

impl GameAssetsEditor {
    pub fn open_houses_window(world: &mut World) {
        let mut houses = houses().clone();
        Window::new("Houses Editor", move |ui, _| {
            for (name, house) in &mut houses {
                CollapsingHeader::new(name).show(ui, |ui| {
                    house.show_mut(None, ui);
                });
            }
        })
        .push(world);
    }
}
