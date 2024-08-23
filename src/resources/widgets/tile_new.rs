use egui::NumExt;

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
            self.max_size.x = self.need_size.x.at_most(space.x);
            space.x -= self.max_size.x;
        }
        if full || self.side.is_y() {
            self.max_size.y = self.need_size.y.at_most(space.y);
            space.y -= self.max_size.y;
        }
    }
    fn show(&mut self, id: Id, focused: bool, ctx: &egui::Context, world: &mut World) {
        let need_size = self.need_size.at_most(self.max_size);
        self.size += (if self.side.is_x() {
            need_size.x
        } else {
            need_size.y
        } - self.size)
            * delta_time(world)
            * 5.0;
        let frame = if focused {
            FRAME.stroke(Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            })
        } else {
            FRAME
        };
        let content = |ui: &mut Ui| {
            let all_rect = ui.available_rect_before_wrap();
            ui.set_clip_rect(all_rect);
            let ui = &mut ui.child_ui(all_rect, Layout::top_down(Align::Min), None);
            let before = ui.min_rect().max;
            (self.content)(ui, world);
            self.need_size = ui.min_rect().max - before;
        };
        match self.side {
            Side::Right => SidePanel::right(id)
                .frame(frame)
                .exact_width(self.size)
                .resizable(false)
                .show_separator_line(false)
                .show(ctx, content),
            Side::Left => SidePanel::left(id)
                .frame(frame)
                .exact_width(self.size)
                .resizable(false)
                .show_separator_line(false)
                .show(ctx, content),
            Side::Top => TopBottomPanel::top(id)
                .frame(frame)
                .exact_height(self.size)
                .resizable(false)
                .show_separator_line(false)
                .show(ctx, content),
            Side::Bottom => TopBottomPanel::bottom(id)
                .frame(frame)
                .exact_height(self.size)
                .resizable(false)
                .show_separator_line(false)
                .show(ctx, content),
        };
    }

    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        // SidePanel::right("qwe")
        //     .frame(FRAME)
        //     .show(ctx, |ui| "qwetqwe qwe qwe".cstr().label(ui));
        // SidePanel::right("asd")
        //     .frame(FRAME)
        //     .exact_width(400.0)
        //     .show(ctx, |ui| {
        //         "asdasdasdasd sda sd asd asd as das d asd asd"
        //             .cstr()
        //             .label(ui)
        //     });
        let mut tr = world.resource_mut::<TileResource>();
        if tr.data.is_empty() {
            return;
        }
        let mut tiles = mem::take(&mut tr.data);
        let focused = tr.focused;
        let mut available_space = ctx.available_rect().size();
        let margin = tiles.len() as f32 * MARGIN * 4.0;
        available_space.x -= margin;
        available_space.y -= margin;
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
