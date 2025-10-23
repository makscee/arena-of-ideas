use super::*;

pub trait NodePublish {
    fn open_publish_window(self, world: &mut World)
    where
        Self: FRecursiveNodeEdit + Node + Clone + 'static;
}

impl<T> NodePublish for T
where
    T: FRecursiveNodeEdit + Node + Clone + 'static,
{
    fn open_publish_window(self, world: &mut World) {
        const WINDOW_ID: &str = "publish_node";
        if WindowPlugin::is_open(WINDOW_ID, world) {
            return;
        }

        let mut node = self.clone();

        Window::new(WINDOW_ID, move |ui, world| {
            ui.heading("Publish Node");
            ui.separator();

            node.render_recursive_edit(ui);

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    WindowPlugin::close_current(world);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Publish").clicked() {
                        let packed = node.pack();
                        let pack_string = packed.to_string();

                        cn().reducers
                            .content_publish_node(pack_string)
                            .notify_error_op();
                        WindowPlugin::close_current(world);
                    }
                });
            });
        })
        .default_width(600.0)
        .default_height(500.0)
        .push(world);
    }
}
