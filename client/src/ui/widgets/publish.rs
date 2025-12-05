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
    fn open_publish_window(self, world: &mut World, mut parent: Option<u64>) {
        let mut node = self.clone();
        let kind = T::kind_s();
        let parent_kind = kind.component_parent();
        let parents = if let Some(parent_kind) = parent_kind {
            match with_incubator_source(|ctx| {
                node_kind_match!(parent_kind, {
                    Ok(ctx
                        .world_mut()?
                        .query::<&NodeType>()
                        .iter(ctx.world()?)
                        .filter(|n| {
                            cn().db
                                .creation_parts()
                                .node_id()
                                .find(&n.id)
                                .map(|cp| todo!())
                                .unwrap_or(true)
                        })
                        .map(|n| (n.id, n.title(ctx)))
                        .collect_vec())
                })
            }) {
                Ok(parents) => parents,
                Err(e) => {
                    e.cstr().notify_error(world);
                    return;
                }
            }
        } else {
            default()
        };
        Confirmation::new("Publish Node")
            .accept_name("Publish")
            .content(move |ui, _world, button_pressed| {
                if parent_kind.is_some() && parent.is_none() {
                    if parents.is_empty() {
                        format!(
                            "[red [b No available parents of {} kind]]",
                            parent_kind.unwrap()
                        )
                        .label(ui);
                        return;
                    }
                    "Select Parent".cstr().label(ui);
                    with_solid_source(|ctx| {
                        parents
                            .as_list(|(_, title), _, ui| title.cstr().label(ui))
                            .with_hover(|(id, _), _, ui| {
                                if "Select".cstr().button(ui).clicked() {
                                    parent = Some(*id);
                                }
                            })
                            .compose(ctx, ui);
                        Ok(())
                    })
                    .ui(ui);
                    return;
                }
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
