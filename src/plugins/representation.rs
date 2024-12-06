use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::update);
    }
}

impl RepresentationPlugin {
    fn update(
        reps: Query<(Entity, &Representation, &Transform), With<NodeState>>,
        states: Query<&NodeState>,
        parents: Query<&Parent>,
        mut painter: ShapePainter,
    ) {
        for (entity, r, t) in &reps {
            match &r.material.t {
                MaterialType::Shape { shape, modifiers } => match shape {
                    Shape::Rectangle { size } => {
                        let size = size
                            .get_value(entity, &states, &parents)
                            .unwrap()
                            .get_vec2()
                            .unwrap();
                        painter.rect(size);
                    }
                    Shape::Circle { radius } => todo!(),
                },
            }
        }
    }
}
