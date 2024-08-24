use bevy::math::FloatExt;
use egui::{Area, NumExt};

use super::*;

pub struct Tile {
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
    side: Side,
    size: f32,
    max_size: f32,
    content_size: egui::Vec2,
    margin_size: egui::Vec2,
    transparent: bool,
}

#[derive(Resource)]
pub struct TileResource {
    tiles: Vec<Tile>,
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

impl Default for TileResource {
    fn default() -> Self {
        Self {
            tiles: default(),
            focused: default(),
        }
    }
}

impl Tile {
    pub fn new(side: Side, content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static) -> Self {
        Self {
            content: Box::new(content),
            side,
            size: 0.0,
            max_size: 0.0,
            content_size: default(),
            margin_size: FRAME.total_margin().sum(),
            transparent: false,
        }
    }
    pub fn push(self, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.tiles.push(self);
        tr.focused = tr.tiles.len() - 1;
    }
    pub fn remove(world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.tiles.pop();
        if !tr.tiles.is_empty() {
            tr.focused = tr.tiles.len() - 1;
        }
    }
    pub fn move_focus(delta: i32, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.focused = (tr.focused as i32 + delta).clamp(0, tr.tiles.len() as i32 - 1) as usize;
    }
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }
    pub fn set_size(mut self, value: f32) -> Self {
        self.size = value;
        self.max_size = value;
        self
    }
    fn take_space(&mut self, full: bool, space: &mut egui::Vec2) {
        if self.side.is_x() {
            self.max_size = self.content_size.x.at_most(space.x);
        } else {
            self.max_size = self.content_size.y.at_most(space.y);
        }
        if full {
            *space -= self.content_size + egui::Vec2::splat(MARGIN * 4.0);
        } else {
            if self.side.is_x() {
                space.x -= self.max_size;
            } else {
                space.y -= self.max_size;
            }
        }
    }
    fn get_screen_rect(ctx: &egui::Context) -> Rect {
        ctx.data(|r| r.get_temp::<Rect>(screen_rect_id())).unwrap()
    }
    fn set_screen_rect(rect: Rect, ctx: &egui::Context) {
        ctx.data_mut(|w| w.insert_temp(screen_rect_id(), rect));
    }
    pub fn show_immediate(
        &mut self,
        id: Id,
        focused: bool,
        ctx: &egui::Context,
        world: &mut World,
    ) {
        let mut frame = if focused {
            FRAME.stroke(Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            })
        } else {
            FRAME
        };
        if self.transparent {
            frame = frame.fill(Color32::TRANSPARENT);
        }
        let screen_rect = Self::get_screen_rect(ctx);
        let (area, rect) = match self.side {
            Side::Right => (
                Area::new(id)
                    .pivot(Align2::RIGHT_TOP)
                    .fixed_pos(screen_rect.right_top()),
                screen_rect.with_min_x(screen_rect.max.x - self.size - self.margin_size.x),
            ),
            Side::Left => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(screen_rect.left_top()),
                screen_rect.with_max_x(screen_rect.min.x + self.size + self.margin_size.x),
            ),
            Side::Top => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(screen_rect.left_top()),
                screen_rect.with_max_y(screen_rect.min.y + self.size + self.margin_size.y),
            ),
            Side::Bottom => (
                Area::new(id)
                    .pivot(Align2::LEFT_BOTTOM)
                    .fixed_pos(screen_rect.left_bottom()),
                screen_rect.with_min_y(screen_rect.max.y - self.size - self.margin_size.y),
            ),
        };

        area.constrain_to(rect).show(ctx, |ui| {
            frame.show(ui, |ui| {
                let rect = rect.shrink2(self.margin_size * 0.5);
                ui.expand_to_include_rect(rect);
                ui.set_clip_rect(rect);
                let ui = &mut ui.child_ui(rect, Layout::top_down(Align::Min), None);
                (self.content)(ui, world);

                self.content_size = ui.min_size();
            });
        });
        let screen_rect = match self.side {
            Side::Right => {
                screen_rect.with_max_x(screen_rect.max.x - self.size - self.margin_size.x)
            }
            Side::Left => {
                screen_rect.with_min_x(screen_rect.min.x + self.size + self.margin_size.x)
            }
            Side::Top => screen_rect.with_min_y(screen_rect.min.y + self.size + self.margin_size.y),
            Side::Bottom => {
                screen_rect.with_max_y(screen_rect.max.y - self.size - self.margin_size.y)
            }
        };
        Self::set_screen_rect(screen_rect, ctx);
    }
    fn show(&mut self, id: Id, focused: bool, ctx: &egui::Context, world: &mut World) {
        let need_size = if self.side.is_x() {
            self.content_size.x
        } else {
            self.content_size.y
        }
        .at_most(self.max_size);
        self.size = self.size.lerp(need_size, delta_time(world) * 8.0);
        self.show_immediate(id, focused, ctx, world);
    }

    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        if tr.tiles.is_empty() {
            return;
        }
        let mut tiles = mem::take(&mut tr.tiles);
        let focused = tr.focused;
        Self::set_screen_rect(ctx.available_rect(), ctx);
        let mut available_space = ctx.available_rect().size();
        for tile in &tiles {
            if tile.side.is_x() {
                available_space.x -= tile.margin_size.x;
            } else {
                available_space.y -= tile.margin_size.y;
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
        tiles.extend(world.resource_mut::<TileResource>().tiles.drain(..));
        let mut tr = world.resource_mut::<TileResource>();
        tr.tiles = tiles;
    }

    pub fn add_team(gid: u64, world: &mut World) {
        Self::new(Side::Right, move |ui, world| {
            gid.get_team().show(ui, world);
        })
        .push(world)
    }
    pub fn add_user(gid: u64, world: &mut World) {
        Self::new(Side::Right, move |ui, world| {
            gid.get_user().show(ui, world);
        })
        .push(world)
    }
    pub fn add_fused_unit(unit: FusedUnit, world: &mut World) {
        Self::new(Side::Right, move |ui, world| {
            unit.show(ui, world);
        })
        .push(world)
    }

    pub fn on_state_changed(to: GameState, world: &mut World) {
        world.resource_mut::<TileResource>().tiles.clear();
        match to {
            GameState::Inbox => Tile::new(Side::Left, |ui, world| {
                Notification::show_all_table(ui, world)
            })
            .push(world),
            GameState::Meta => MetaPlugin::add_tiles(world),
            GameState::Shop => ShopPlugin::add_tiles(world),
            GameState::Battle => BattlePlugin::add_tiles(world),
            GameState::TableView(query) => TableViewPlugin::add_tiles(query, world),
            _ => {}
        }
    }
}

fn screen_rect_id() -> Id {
    static ID: OnceCell<Id> = OnceCell::new();
    *ID.get_or_init(|| Id::new("available_screen_rect"))
}
