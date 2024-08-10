use egui::NumExt;

use super::*;

pub struct TileWidget {
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
    size: f32,
    need_size: f32,
    shrink_size: f32,
}

#[derive(Resource, Default)]
pub struct TileResource {
    data: Vec<TileWidget>,
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(7.0),
    outer_margin: Margin::same(7.0),
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
            shrink_size: default(),
        }
    }
    pub fn push(self, world: &mut World) {
        world.resource_mut::<TileResource>().data.push(self)
    }
    fn show(&mut self, id: Id, ctx: &egui::Context, world: &mut World) -> f32 {
        let all_rect = ctx.available_rect();
        self.size += (self.need_size - self.shrink_size - self.size) * delta_time(world) * 5.0;

        let size = self.size.at_most(all_rect.width() - 28.0);
        SidePanel::left(id)
            .show_separator_line(false)
            .frame(FRAME)
            .exact_width(size)
            .resizable(false)
            .show(ctx, |ui| {
                let all_rect = ui.available_rect_before_wrap();
                ui.set_clip_rect(all_rect);
                let ui = &mut ui.child_ui(all_rect, Layout::top_down(Align::Min));
                let before = ui.min_rect().width();
                (self.content)(ui, world);
                self.need_size = ui.min_rect().width() - before;
            });
        self.size - size
    }

    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        let mut tiles = mem::take(&mut world.resource_mut::<TileResource>().data);
        let len = tiles.len();
        let mut need = 0.0;
        let mut coming = 0.0;
        for (i, tile) in tiles.iter_mut().enumerate() {
            let overflow = tile.show(Id::new("tile").with(i), ctx, world);
            if i == len - 1 && overflow > 0.0 {
                need = overflow;
            }
            if tile.size > tile.need_size - tile.shrink_size {
                coming += tile.size - tile.need_size + tile.shrink_size;
            }
        }
        need -= coming;
        if need > 0.0 {
            dbg!(need);
            for tile in tiles.iter_mut() {
                let shrink = (tile.need_size - tile.shrink_size - 0.0).clamp(0.0, need);
                need -= shrink;
                tile.shrink_size += shrink;
            }
        }
        tiles.extend(world.resource_mut::<TileResource>().data.drain(..));
        world.resource_mut::<TileResource>().data = tiles;
    }
}
