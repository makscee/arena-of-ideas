use bevy::color::Alpha;

use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (Self::update, Self::hover));
    }
}

impl RepresentationPlugin {
    fn hover(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        let p = CameraPlugin::pixel_unit(ctx, world) * 2.0;
        let mut open_window = None;
        let mut close_window = None;
        for (unit, t) in world.query::<(&Unit, &GlobalTransform)>().iter(world) {
            let pos = world_to_screen(t.translation(), world).to_pos2();
            if ctx.rect_contains_pointer(
                egui::LayerId::background(),
                Rect::from_center_size(pos, egui::vec2(p, p)),
            ) {
                if left_mouse_just_pressed(world) {
                    if WindowPlugin::is_open(&unit.name, world) {
                        close_window = Some(unit.name.clone());
                    } else {
                        open_window = Some((unit.entity.unwrap(), unit.name.clone()));
                    }
                }
                cursor_window(ctx, |ui| {
                    unit.ui(0, ui, world);
                });
            }
        }
        if let Some(name) = close_window {
            WindowPlugin::close(&name, world);
        }
        if let Some((entity, name)) = open_window {
            Window::new(name, move |ui, world| {
                if let Some(unit) = world.get::<Unit>(entity) {
                    unit.ui(0, ui, world);
                }
            })
            .no_frame()
            .push(world);
        }
    }
    fn update(
        reps: Query<(Entity, &Representation), With<NodeState>>,
        context: StateQuery,
        mut egui_context: Query<&mut EguiContext>,
        camera: Query<(&Camera, &GlobalTransform)>,
        mut painter: ShapePainter,
    ) {
        let ctx = egui_context.single_mut().into_inner().get_mut();
        let cam = camera.single();
        let mut context = Context::new(context);
        for (e, r) in &reps {
            context.set_owner(e);
            let count = r.material.count.at_least(1) as i32;
            for m in &r.material.modifiers {
                match m.apply(e, &mut context) {
                    Ok(_) => {}
                    Err(e) => error!("RModifier error: {e}"),
                }
            }
            for i in 0..count {
                context.set_var(VarName::index, i.into());
                match Self::paint(e, &r.material, &context, ctx, cam, &mut painter) {
                    Ok(_) => {}
                    Err(e) => error!("Paint error: {e}"),
                }
            }
            context.clear();
            painter.reset();
        }
    }
    fn paint(
        e: Entity,
        m: &RMaterial,
        context: &Context,
        ctx: &egui::Context,
        cam: (&Camera, &GlobalTransform),
        painter: &mut ShapePainter,
    ) -> Result<(), ExpressionError> {
        let pos = context.get_var(VarName::position).to_e()?.get_vec2()?
            + context
                .get_var(VarName::offset)
                .and_then(|v| v.get_vec2().ok())
                .unwrap_or_default();
        match &m.t {
            MaterialType::Shape { shape, modifiers } => {
                for m in modifiers {
                    match m.apply(painter, e, context) {
                        Ok(_) => {}
                        Err(e) => error!("Modifier error: {e}"),
                    }
                }
                painter.set_translation(pos.extend(0.0));
                match shape {
                    Shape::Rectangle { size } => {
                        painter.rect(size.get_vec2(e, context)?);
                    }
                    Shape::Circle { radius } => {
                        painter.circle(radius.get_f32(e, context)?);
                    }
                };
            }
            MaterialType::Text { text } => {
                let index = context.get_var(VarName::index).to_e()?;
                let pos = world_to_screen_cam(pos.extend(0.0), &cam.0, &cam.1);
                let text = text.get_string(e, context)?;
                let color = context.get_var(VarName::color).to_e()?.get_color()?.c32();
                Area::new(Id::new(e).with(index))
                    .pivot(Align2::CENTER_CENTER)
                    .fixed_pos(pos.to_pos2())
                    .show(ctx, |ui| {
                        text.cstr_c(color).label(ui);
                    });
            }
        }
        Ok(())
    }
}

trait RModifierApply {
    fn apply(&self, e: Entity, context: &mut Context) -> Result<(), ExpressionError>;
}

impl RModifierApply for RModifier {
    fn apply(&self, e: Entity, context: &mut Context) -> Result<(), ExpressionError> {
        match self {
            RModifier::Color(x) => context.set_var(VarName::color, x.get_value(e, context)?),
            RModifier::Offset(x) => context.set_var(VarName::offset, x.get_value(e, context)?),
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
