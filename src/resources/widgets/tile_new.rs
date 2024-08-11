use egui::NumExt;

use super::*;

pub struct TileWidget {
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
    size: f32,
    need_size: f32,
    max_size: f32,
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
    pub fn new(content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static) -> Self {
        Self {
            content: Box::new(content),
            size: 0.0,
            need_size: 20.0,
            max_size: f32::MAX,
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
    fn show(&mut self, id: Id, focused: bool, ctx: &egui::Context, world: &mut World) {
        let need_size = self.need_size.at_most(self.max_size);
        self.size += (need_size - self.size) * delta_time(world) * 5.0;
        let frame = if focused {
            FRAME.stroke(Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            })
        } else {
            FRAME
        };
        SidePanel::left(id)
            .show_separator_line(false)
            .frame(frame)
            .exact_width(self.size)
            .resizable(false)
            .show(ctx, |ui| {
                let all_rect = ui.available_rect_before_wrap();
                ui.set_clip_rect(all_rect);
                let ui = &mut ui.child_ui(all_rect, Layout::top_down(Align::Min));
                let before = ui.min_rect().width();
                (self.content)(ui, world);
                self.need_size = ui.min_rect().width() - before;
            });
    }

    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        if tr.data.is_empty() {
            return;
        }
        let mut tiles = mem::take(&mut tr.data);
        let focused = tr.focused;
        let mut available_size = ctx.available_rect().width();
        available_size -= tiles.len() as f32 * MARGIN * 4.0;
        let max_size_focused = tiles[focused].need_size.at_most(available_size);
        tiles[focused].max_size = max_size_focused;
        available_size -= max_size_focused;
        let (left, right) = tiles.split_at_mut(focused);
        let mut left_i = left.len() as i32 - 1;
        let mut right_i = 1;
        let l = left.len().max(right.len());
        for _ in 0..=l {
            if left_i >= 0 {
                if let Some(tile) = left.get_mut(left_i as usize) {
                    let max_size = tile.need_size.at_most(available_size);
                    tile.max_size = max_size;
                    available_size -= max_size;
                    left_i -= 1;
                }
            }
            if let Some(tile) = right.get_mut(right_i) {
                let max_size = tile.need_size.at_most(available_size);
                tile.max_size = max_size;
                available_size -= max_size;
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
