use super::*;

pub trait NodePublish {
    fn open_publish_window(self, world: &mut World, parent: Option<u64>)
    where
        Self: FRecursiveNodeEdit + Node + Clone + 'static;
}

impl<T> NodePublish for T
where
    T: FRecursiveNodeEdit + Node + Clone + 'static,
{
    fn open_publish_window(self, world: &mut World, parent: Option<u64>) {
        let mut node = self.clone();

        Confirmation::new("Publish Node")
            .accept_name("Publish")
            .content(move |ui, _world, button_pressed| {
                ui.vertical(|ui| {
                    node.render_recursive_edit(ui);
                });

                if let Some(true) = button_pressed {
                    let packed = node.pack();
                    let pack_string = packed.to_string();
                    cn().reducers
                        .content_publish_node(pack_string, parent)
                        .notify_error_op();
                }
            })
            .push(world);
    }
}
