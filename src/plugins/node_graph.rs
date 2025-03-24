use bevy::ecs::event::EventReader;

use super::*;

pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::read_events)
            .init_resource::<GraphData>();
    }
}

#[derive(Component)]
struct GraphNode {
    pos: Pos2,
    node: TNode,
}

#[derive(Resource, Default)]
struct GraphData {
    selected_node: Option<u64>,
}

impl NodeGraphPlugin {
    fn show_node(node: &GraphNode, entity: Entity, parent_rect: Rect, ui: &mut Ui, world: &World) {
        let selected_id = world.resource::<GraphData>().selected_node;
        ui.horizontal(|ui| {
            if node
                .node
                .kind()
                .data_frame_ui(
                    entity,
                    selected_id.is_some_and(|id| id == node.node.id),
                    ui,
                    world,
                )
                .name_clicked()
            {
                let id = node.node.id;
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
                let Some(children) = world.get::<Children>(entity) else {
                    return;
                };
                for (entity, child) in children
                    .into_iter()
                    .filter_map(|c| world.get::<GraphNode>(*c).map(|n| (*c, n)))
                    .sorted_by_key(|(_, n)| (&n.node.kind, n.node.id))
                {
                    Self::show_node(child, entity, rect, ui, world);
                    if selected_id.is_some_and(|id| id == child.node.id) {
                        let potential_links = cn()
                            .db
                            .nodes_world()
                            .iter()
                            .filter_map(|n| {
                                if n.parent == ID_INCUBATOR && n.kind == child.node.kind {
                                    Some((
                                        cn().db
                                            .incubator_links()
                                            .iter()
                                            .find_map(|l| {
                                                let source = cn()
                                                    .db
                                                    .incubator_source()
                                                    .node_id()
                                                    .find(&node.node.id)?
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
        let Some(all) = All::get_by_id(ID_ALL, world).map(|n| n.entity()) else {
            return;
        };
        let Some(node) = world.get::<GraphNode>(all) else {
            return;
        };
        Self::show_node(node, all, Rect::ZERO, ui, world);
    }
    fn read_events(mut events: EventReader<StdbEvent>) {
        if events.is_empty() {
            return;
        }
        for StdbEvent {
            entity,
            node,
            change,
        } in events.read()
        {
            let entity = *entity;
            match change {
                StdbChange::Insert => {
                    let node = node.clone();
                    if node.parent == ID_INCUBATOR || node.id == ID_INCUBATOR {
                        continue;
                    }
                    op(move |world| {
                        world.entity_mut(entity).insert(GraphNode {
                            pos: pos2(thread_rng().gen(), thread_rng().gen()) * 10.0,
                            node,
                        });
                    });
                }
                _ => {}
            }
        }
    }
}
