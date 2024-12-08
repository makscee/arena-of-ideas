use bevy::color::ColorToPacked;
use chrono::Utc;

use super::*;

pub fn world_to_screen(pos: Vec3, world: &World) -> Vec2 {
    let entity = world.entity(world.resource::<CameraData>().entity);
    let camera = entity.get::<Camera>().unwrap();
    let transform = entity.get::<GlobalTransform>().unwrap();
    camera.world_to_viewport(transform, pos).unwrap_or_default()
}
pub fn world_to_screen_cam(pos: Vec3, cam: &Camera, cam_transform: &GlobalTransform) -> Vec2 {
    cam.world_to_viewport(cam_transform, pos)
        .unwrap_or_default()
}
pub fn screen_to_world(pos: Vec2, world: &World) -> Vec2 {
    let entity = CameraPlugin::entity(world);
    let camera = world.get::<Camera>(entity).unwrap();
    let transform = world.get::<GlobalTransform>(entity).unwrap();
    screen_to_world_cam(pos, camera, transform)
}
pub fn screen_to_world_cam(pos: Vec2, cam: &Camera, cam_transform: &GlobalTransform) -> Vec2 {
    cam.viewport_to_world_2d(cam_transform, pos)
        .unwrap_or_default()
}
pub fn cursor_world_pos(world: &mut World) -> Option<Vec2> {
    cursor_pos(world).map(|p| screen_to_world(p, world))
}
pub fn draw_curve(
    p1: Pos2,
    p2: Pos2,
    p3: Pos2,
    p4: Pos2,
    width: f32,
    color: Color32,
    arrow: bool,
    ui: &mut Ui,
) {
    let points = [p1, p2, p3, p4];
    let stroke = Stroke { width, color };
    let curve = egui::Shape::CubicBezier(egui::epaint::CubicBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
    ui.painter().add(curve);
    if !arrow {
        return;
    }
    let t = p4.to_vec2();
    let t1 = (p3.to_vec2() - t).normalized() * 15.0;
    let p1 = (t + t1 + t1.rot90()).to_pos2();
    let p2 = (t + t1 - t1.rot90()).to_pos2();
    let points = [p1, p4, p2];
    let arrow = egui::Shape::Path(PathShape::line(points.into(), stroke));
    ui.painter().add(arrow);
}
pub fn global_settings() -> GlobalSettings {
    if is_connected() {
        todo!()
    } else {
        todo!()
    }
}
pub fn app_exit(world: &mut World) {
    world
        .get_resource_mut::<bevy::prelude::Events<bevy::app::AppExit>>()
        .unwrap()
        .send(bevy::app::AppExit::Success);
}
pub fn app_exit_op() {
    OperationsPlugin::add(app_exit)
}
pub fn cur_state(world: &World) -> GameState {
    *world.resource::<State<GameState>>().get()
}
pub fn debug_rect(rect: Rect, ctx: &egui::Context) {
    ctx.debug_painter().rect(
        rect,
        Rounding::ZERO,
        YELLOW_DARK.gamma_multiply(0.5),
        Stroke {
            width: 1.0,
            color: YELLOW,
        },
    );
}
pub fn debug_available_rect(ui: &mut Ui) {
    debug_rect(ui.available_rect_before_wrap(), ui.ctx());
}
pub fn can_afford(cost: i64) -> bool {
    cn().db.wallet().current().amount >= cost
}
pub fn show_daily_refresh_timer(ui: &mut Ui) {
    let now = Utc::now().timestamp();
    let til_refresh = (now / 86400 + 1) * 86400 - now;
    format!(
        "Refresh in {}",
        format_duration(til_refresh as u64).cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold)
    )
    .label(ui);
}
pub fn rng_seeded(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

pub trait ToBVec2 {
    fn to_bvec2(&self) -> Vec2;
}

impl ToBVec2 for Pos2 {
    fn to_bvec2(&self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

pub trait ToPos2 {
    fn to_pos2(&self) -> Pos2;
    fn to_evec2(&self) -> egui::Vec2;
}

impl ToPos2 for Vec2 {
    fn to_pos2(&self) -> Pos2 {
        pos2(self.x, self.y)
    }
    fn to_evec2(&self) -> egui::Vec2 {
        egui::vec2(self.x, self.y)
    }
}

pub trait ToColor {
    fn to_color(&self) -> Color;
}

impl ToColor for Color32 {
    fn to_color(&self) -> Color {
        let a = self.to_array();
        Color::srgba_u8(a[0], a[1], a[2], a[3])
    }
}

pub trait ToC32 {
    fn c32(&self) -> Color32;
}

impl ToC32 for Color {
    fn c32(&self) -> Color32 {
        let c = self.to_srgba().to_u8_array();
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }
}

pub trait ToCustomColor {
    fn to_custom_color(&self) -> CustomColor;
}

impl ToCustomColor for Color32 {
    fn to_custom_color(&self) -> CustomColor {
        let a = self.to_array();
        CustomColor::new(a[0], a[1], a[2])
    }
}

pub trait WorldExt {
    fn game_clear(&mut self);
}

impl WorldExt for World {
    fn game_clear(&mut self) {
        clear_entity_names();
    }
}

pub fn name_color(_: &str) -> Color32 {
    MISSING_COLOR
}
