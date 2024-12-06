use bevy::{color::Alpha, ecs::system::SystemParam};

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
        context: StateQuery,
        mut painter: ShapePainter,
    ) {
        let mut context = Context::new(context);
        for (e, r) in &reps {
            context.set_owner(e);
            let count = r.material.count.at_least(1) as i32;
            for i in 0..count {
                context.set_var(VarName::index, i.into());
                match Self::paint(e, &r.material, &context, &mut painter) {
                    Ok(_) => {}
                    Err(e) => error!("Paint error: {e}"),
                }
            }
            context.clear();
        }
    }
    fn paint(
        e: Entity,
        m: &RMaterial,
        context: &Context,
        painter: &mut ShapePainter,
    ) -> Result<(), ExpressionError> {
        match &m.t {
            MaterialType::Shape { shape, modifiers } => {
                for m in modifiers {
                    match m.apply(painter, e, context) {
                        Ok(_) => {}
                        Err(e) => error!("Modifier error: {e}"),
                    }
                }
                match shape {
                    Shape::Rectangle { size } => {
                        painter.rect(size.get_vec2(e, context)?);
                    }
                    Shape::Circle { radius } => {
                        painter.circle(radius.get_f32(e, context)?);
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
        context: &Context,
    ) -> Result<(), ExpressionError>;
}

impl ShapeModifierApply for ShapeModifier {
    fn apply(
        &self,
        painter: &mut ShapePainter,
        e: Entity,
        context: &Context,
    ) -> Result<(), ExpressionError> {
        match self {
            ShapeModifier::Rotation(v) => painter.rotate_z(v.get_f32(e, context)?),
            ShapeModifier::Scale(v) => painter.scale(v.get_vec2(e, context)?.extend(1.0)),
            ShapeModifier::Color(v) => painter.set_color(v.get_color(e, context)?),
            ShapeModifier::Hollow(v) => painter.hollow = v.get_bool(e, context)?,
            ShapeModifier::Thickness(v) => painter.thickness = v.get_f32(e, context)?,
            ShapeModifier::Roundness(v) => painter.roundness = v.get_f32(e, context)?,
            ShapeModifier::Alpha(v) => painter.color.set_alpha(v.get_f32(e, context)?),
        };
        Ok(())
    }
}
