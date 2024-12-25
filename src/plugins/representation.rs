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
        let mut open_window = None;
        let mut close_window = None;
        let up = unit_pixels();
        for (unit, t) in world.query::<(&Unit, &GlobalTransform)>().iter(world) {
            let pos = world_to_screen(t.translation(), world).to_pos2();
            if !ctx.screen_rect().contains(pos) {
                continue;
            }
            let rect = Rect::from_center_size(pos, egui::vec2(up, up));
            let resp = Area::new(Id::new(&unit.entity))
                .fixed_pos(rect.center())
                .pivot(Align2::CENTER_CENTER)
                .constrain(false)
                .sense(Sense::click())
                .show(ctx, |ui| {
                    ui.expand_to_include_rect(rect);
                })
                .response;
            if resp.hovered() {
                cursor_window(ctx, |ui| {
                    unit.ui(
                        0,
                        Context::new_world(world).set_owner(unit.entity.unwrap()),
                        ui,
                    );
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
                    unit.ui(
                        0,
                        Context::new_world(world).set_owner(unit.entity.unwrap()),
                        ui,
                    );
                }
            })
            .no_frame()
            .push(world);
        }
    }
    fn update(
        reps: Query<(Entity, &Representation), With<NodeState>>,
        state: StateQuery,
        mut egui_context: Query<&mut EguiContext>,
        camera: Query<(&Camera, &GlobalTransform)>,
    ) {
        let ctx = egui_context.single_mut().into_inner().get_mut();
        let cam = camera.single();
        let mut context = Context::new(&state);
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
        m: &Material,
        context: &Context,
        ctx: &egui::Context,
        cam: (&Camera, &GlobalTransform),
    ) -> Result<(), ExpressionError> {
        let pos = context.get_var(VarName::position)?.get_vec2()?
            + context
                .get_var(VarName::offset)
                .and_then(|v| v.get_vec2())
                .unwrap_or_default();
        let pos = world_to_screen_cam(pos.extend(0.0), &cam.0, &cam.1).to_pos2();
        let size = unit_pixels() * 2.1;
        let size = egui::vec2(size, size);
        let rect = Rect::from_center_size(pos, size);
        if !ctx.screen_rect().intersects(rect) {
            return Ok(());
        }
        let owner = context.get_owner().unwrap();
        Area::new(Id::new(owner))
            .constrain(false)
            .fixed_pos(rect.center())
            .pivot(Align2::CENTER_CENTER)
            .order(Order::Background)
            .show(ctx, |ui| Self::paint_rect(rect, context, m, ui))
            .inner
    }
    pub fn paint_rect(
        rect: Rect,
        context: &Context,
        m: &Material,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        if !context
            .get_var(VarName::visible)
            .and_then(|v| v.get_bool())
            .unwrap_or(true)
        {
            return Ok(());
        }
        let mut p = Painter::new(rect, ui.ctx());
        for a in &m.0 {
            match a.paint(context, &mut p, ui) {
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
        }
        PainterAction::Paint.paint(context, &mut p, ui)?;
        Ok(())
    }
}
