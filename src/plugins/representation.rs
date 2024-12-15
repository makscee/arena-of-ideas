use bevy::color::Alpha;
use egui::{paint_texture_at, Mesh};
use epaint::{TessellationOptions, Tessellator};

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
            let rect = Rect::from_center_size(pos, egui::vec2(p, p));
            let resp = Area::new(Id::new(&unit.entity))
                .constrain_to(rect)
                .sense(Sense::click())
                .show(ctx, |ui| {
                    ui.expand_to_include_rect(rect);
                })
                .response;
            if resp.hovered() {
                cursor_window(ctx, |ui| {
                    unit.ui(0, ui, world);
                });
            }
            if resp.clicked() {
                if WindowPlugin::is_open(&unit.name, world) {
                    close_window = Some(unit.name.clone());
                } else {
                    open_window = Some((unit.entity.unwrap(), unit.name.clone()));
                }
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
    ) {
        let ctx = egui_context.single_mut().into_inner().get_mut();
        let cam = camera.single();
        let mut context = Context::new(context);
        for (e, r) in &reps {
            context.set_owner(e);
            match Self::paint(&r.material, &context, ctx, cam) {
                Ok(_) => {}
                Err(e) => error!("Paint error: {e}"),
            };
            context.clear();
        }
    }
    fn paint(
        m: &RMaterial,
        context: &Context,
        ctx: &egui::Context,
        cam: (&Camera, &GlobalTransform),
    ) -> Result<(), ExpressionError> {
        let pos = context.get_var(VarName::position).to_e()?.get_vec2()?
            + context
                .get_var(VarName::offset)
                .and_then(|v| v.get_vec2().ok())
                .unwrap_or_default();
        let pos = world_to_screen_cam(pos.extend(0.0), &cam.0, &cam.1);
        let size = unit_pixels() * 2.0;
        let size = egui::vec2(size, size);
        let fonts_size = ctx.fonts(|r| r.texture_atlas().lock().size());
        let mut p = Painter {
            rect: Rect::from_center_size(pos.to_pos2(), size),
            color: VISIBLE_LIGHT,
            mesh: Mesh::default(),
            tesselator: Tessellator::new(
                unit_pixels(),
                TessellationOptions::default(),
                fonts_size,
                default(),
            ),
        };
        let owner = context.get_owner().unwrap();
        Area::new(Id::new(owner))
            .constrain_to(p.rect)
            .show(ctx, |ui| {
                for a in &m.actions {
                    match a.paint(context, &mut p, ui) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                Ok(())
            })
            .inner
    }
}
