use bevy::ecs::event::EventReader;
use bevy_egui::egui::UiBuilder;

use super::*;

pub struct NodeGraphPlugin;

impl Plugin for NodeGraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (Self::read_events, Self::update))
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
    camera_pos: egui::Vec2,
}

impl GraphNode {
    fn show(&self, rect: Rect, pos: Pos2, entity: Entity, ui: &mut Ui, world: &World) {
        let ui = &mut ui.new_child(UiBuilder::new().max_rect(Rect::from_pos(pos).expand(10.0)));
        self.node.kind().show(entity, ui, world);
    }
}

impl NodeGraphPlugin {
    fn update(mut query: Query<(Entity, Option<&Parent>, &mut GraphNode)>) {
        let desired_dist = 200.0;
        let mut changes: Vec<(Entity, egui::Vec2)> = default();
        for (entity, parent, node) in query.iter() {
            let Some(parent) = parent else {
                continue;
            };
            let (_, _, parent) = query.get(parent.get()).unwrap();
            let delta = parent.pos - node.pos;
            let need_delta = delta.normalized() * (delta.length() - desired_dist);
            changes.push((entity, need_delta));
        }
        let dt = gt().last_delta() * 10.0;
        for (entity, delta) in changes {
            query.get_mut(entity).unwrap().2.pos += delta * dt;
        }
    }
    pub fn pane_ui(ui: &mut Ui, world: &mut World) {
        let pos = world.resource::<GraphData>().camera_pos;
        let rect = ui.available_rect_before_wrap();
        for (entity, node) in world.query::<(Entity, &GraphNode)>().iter(world) {
            let pos = node.pos + rect.center().to_vec2() - pos;
            if rect.contains(pos) {
                node.show(rect, pos, entity, ui, world);
            }
        }
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
