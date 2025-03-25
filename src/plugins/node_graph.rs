use super::*;

pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphData>();
    }
}

#[derive(Resource, Default)]
struct GraphData {
    selected_node: Option<u64>,
}

impl NodeGraphPlugin {
    fn show_node(node: &TNode, parent_rect: Rect, ui: &mut Ui, world: &World) {
        let Some(entity) = world.get_id_link(node.id) else {
            return;
        };
        let selected_id = world.resource::<GraphData>().selected_node;
        ui.horizontal(|ui| {
            if node
                .kind()
                .data_frame_ui(
                    entity,
                    selected_id.is_some_and(|id| id == node.id),
                    ui,
                    world,
                )
                .name_clicked()
            {
                let id = node.id;
                op(move |world| {
                    world.resource_mut::<GraphData>().selected_node = Some(id);
                });
            }
            let rect = ui.min_rect();
            ui.add_space(4.0);
            ui.painter().line(
                [parent_rect.right_center(), rect.left_center()].into(),
                Stroke::new(1.0, tokens_global().ui_element_border_and_focus_rings()),
            );
            ui.vertical(|ui| {
                for child in cn()
                    .db
                    .nodes_world()
                    .iter()
                    .filter(|n| n.parent == node.id && n.parent != n.id)
                    .sorted_by_key(|n| (n.kind(), n.id))
                {
                    Self::show_node(&child, rect, ui, world);
                    if selected_id.is_some_and(|id| id == child.id) {
                        let potential_links = cn()
                            .db
                            .nodes_world()
                            .iter()
                            .filter_map(|n| {
                                if n.parent == ID_INCUBATOR && n.kind == child.kind {
                                    Some((
                                        cn().db
                                            .incubator_links()
                                            .iter()
                                            .find_map(|l| {
                                                let source = cn()
                                                    .db
                                                    .incubator_source()
                                                    .node_id()
                                                    .find(&node.id)?
                                                    .incubator_id;
                                                if l.from == source && l.to == n.id {
                                                    Some(l.score as i32)
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap_or_default(),
                                        n,
                                    ))
                                } else {
                                    None
                                }
                            })
                            .sorted_by_key(|(score, _)| -*score)
                            .collect_vec();
                        for (score, node) in potential_links {
                            ui.horizontal(|ui| {
                                format!("[th [b {score}]]").cstr().label(ui);
                                node.kind()
                                    .show(world.get_id_link(node.id).unwrap(), ui, world);
                            });
                        }
                    }
                }
            })
        });
    }
    pub fn pane_ui(ui: &mut Ui, world: &mut World) {
        let Some(all) = All::get_by_id(ID_ALL, world).map(|n| n.id()) else {
            return;
        };
        let Some(node) = TNode::find(all) else {
            return;
        };
        Self::show_node(&node, Rect::ZERO, ui, world);
    }
}
