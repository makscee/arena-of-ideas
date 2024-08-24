use egui::{Area, NumExt};

use super::*;

pub struct TileWidget {
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
    side: Side,
    size: f32,
    need_size: egui::Vec2,
    max_size: egui::Vec2,
}

#[derive(Resource, Default)]
pub struct TileResource {
    data: Vec<TileWidget>,
    focused: usize,
}

const MARGIN: f32 = 7.0;
const FRAME: Frame = Frame {
    inner_margin: Margin::same(MARGIN),
    outer_margin: Margin::same(MARGIN),
    rounding: Rounding::same(13.0),
    shadow: Shadow::NONE,
    fill: BG_DARK,
    stroke: Stroke::NONE,
};

impl TileWidget {
    pub fn new(side: Side, content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static) -> Self {
        Self {
            content: Box::new(content),
            side,
            size: default(),
            need_size: default(),
            max_size: default(),
        }
    }
    pub fn push(self, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.data.push(self);
        tr.focused = tr.data.len() - 1;
    }
    pub fn remove(world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.data.pop();
        if !tr.data.is_empty() {
            tr.focused = tr.data.len() - 1;
        }
    }
    pub fn move_focus(delta: i32, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.focused = (tr.focused as i32 + delta).clamp(0, tr.data.len() as i32 - 1) as usize;
    }
    fn take_space(&mut self, full: bool, space: &mut egui::Vec2) {
        if full || self.side.is_x() {
            self.max_size.x = (self.need_size.x - MARGIN * 2.0).clamp(0.0, space.x);
            space.x -= self.max_size.x;
        }
        if full || self.side.is_y() {
            self.max_size.y = (self.need_size.y - MARGIN * 2.0).clamp(0.0, space.y);
            space.y -= self.max_size.y;
        }
        self.max_size += egui::Vec2::splat(MARGIN * 2.0);
    }
    fn get_screen_rect(ctx: &egui::Context) -> Rect {
        ctx.data(|r| r.get_temp::<Rect>(screen_rect_id())).unwrap()
    }
    fn set_screen_rect(rect: Rect, ctx: &egui::Context) {
        ctx.data_mut(|w| w.insert_temp(screen_rect_id(), rect));
    }
    fn show(&mut self, id: Id, focused: bool, ctx: &egui::Context, world: &mut World) {
        let need_size = self.need_size.at_most(self.max_size);
        self.size += (if self.side.is_x() {
            need_size.x
        } else {
            need_size.y
        } - self.size)
            * delta_time(world)
            * 8.0;
        let frame = if focused {
            FRAME.stroke(Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            })
        } else {
            FRAME
        };
        let screen_rect = Self::get_screen_rect(ctx);
        let (area, rect) = match self.side {
            Side::Right => (
                Area::new(id)
                    .pivot(Align2::RIGHT_TOP)
                    .fixed_pos(screen_rect.right_top()),
                screen_rect.with_min_x(screen_rect.max.x - self.size),
            ),
            Side::Left => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(screen_rect.left_top()),
                screen_rect.with_max_x(screen_rect.min.x + self.size),
            ),
            Side::Top => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(screen_rect.left_top()),
                screen_rect.with_max_y(screen_rect.min.y + self.size),
            ),
            Side::Bottom => (
                Area::new(id)
                    .pivot(Align2::LEFT_BOTTOM)
                    .fixed_pos(screen_rect.left_bottom()),
                screen_rect.with_min_y(screen_rect.max.y - self.size),
            ),
        };
        area.constrain_to(rect).show(ctx, |ui| {
            frame.show(ui, |ui| {
                let rect = rect.shrink(MARGIN * 2.0);
                ui.expand_to_include_rect(rect);
                ui.set_clip_rect(rect);
                let ui = &mut ui.child_ui(rect, Layout::top_down(Align::Min), None);
                "test test".cstr().label(ui);
                "test test test test test test test test test test"
                    .cstr()
                    .label(ui);
                // "test test".cstr().label(ui);

                self.need_size = ui.min_size()
                    + egui::Vec2::splat(MARGIN * 4.0) * (if focused { 2.0 } else { 1.0 });
            });
        });
        let screen_rect = match self.side {
            Side::Right => screen_rect.with_max_x(screen_rect.max.x - self.size),
            Side::Left => screen_rect.with_min_x(screen_rect.min.x + self.size),
            Side::Top => screen_rect.with_min_y(screen_rect.min.y + self.size),
            Side::Bottom => screen_rect.with_max_y(screen_rect.max.y - self.size),
        };
        Self::set_screen_rect(screen_rect, ctx);
    }

    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        if tr.data.is_empty() {
            return;
        }
        let mut tiles = mem::take(&mut tr.data);
        let focused = tr.focused;
        Self::set_screen_rect(ctx.available_rect(), ctx);
        let mut available_space = ctx.available_rect().size();
        let margin = MARGIN * 2.0;
        available_space -= egui::Vec2::splat(MARGIN * 4.0);
        for tile in &tiles {
            if tile.side.is_x() {
                available_space.x -= margin;
            } else {
                available_space.y -= margin;
            }
        }
        tiles[focused].take_space(true, &mut available_space);
        let (left, right) = tiles.split_at_mut(focused);
        let mut left_i = left.len() as i32 - 1;
        let mut right_i = 1;
        let l = left.len().max(right.len());
        for _ in 0..=l {
            if left_i >= 0 {
                if let Some(tile) = left.get_mut(left_i as usize) {
                    tile.take_space(false, &mut available_space);
                    left_i -= 1;
                }
            }
            if let Some(tile) = right.get_mut(right_i) {
                tile.take_space(false, &mut available_space);
                right_i += 1;
            }
        }

        for (i, tile) in tiles.iter_mut().enumerate() {
            let focused = focused == i;
            tile.show(Id::new("tile").with(i), focused, ctx, world);
        }
        tiles.extend(world.resource_mut::<TileResource>().data.drain(..));
        world.resource_mut::<TileResource>().data = tiles;
    }
}

fn screen_rect_id() -> Id {
    static ID: OnceCell<Id> = OnceCell::new();
    *ID.get_or_init(|| Id::new("available_screen_rect"))
}
