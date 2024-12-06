use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::update);
    }
}

impl RepresentationPlugin {
    fn update(
        reps: Query<(Entity, &Representation), With<NodeState>>,
        s: Query<&NodeState>,
        p: Query<&Parent>,
        mut painter: ShapePainter,
    ) {
        for (e, r) in &reps {
            match Self::paint(e, r, &s, &p, &mut painter) {
                Ok(_) => {}
                Err(e) => error!("Paint error: {e}"),
            }
        }
    }
    fn paint(
        e: Entity,
        r: &Representation,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
        painter: &mut ShapePainter,
    ) -> Result<(), ExpressionError> {
        match &r.material.t {
            MaterialType::Shape { shape, modifiers } => {
                for m in modifiers {
                    match m.apply(painter, e, &s, &p) {
                        Ok(_) => {}
                        Err(e) => error!("Modifier error: {e}"),
                    }
                }
                match shape {
                    Shape::Rectangle { size } => {
                        painter.rect(size.get_vec2(e, &s, &p)?);
                    }
                    Shape::Circle { radius } => {
                        painter.circle(radius.get_f32(e, s, p)?);
                    }
                };
            }
        }
        Ok(())
    }
}

trait ShapeModifierApply {
    fn apply(
        &self,
        painter: &mut ShapePainter,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<(), ExpressionError>;
}

impl ShapeModifierApply for ShapeModifier {
    fn apply(
        &self,
        painter: &mut ShapePainter,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<(), ExpressionError> {
        match self {
            ShapeModifier::Rotation(v) => painter.rotate_z(v.get_f32(e, s, p)?),
            ShapeModifier::Scale(v) => painter.scale(v.get_vec2(e, s, p)?.extend(1.0)),
            ShapeModifier::Color(v) => painter.set_color(v.get_color(e, s, p)?),
        };
        Ok(())
    }
}
