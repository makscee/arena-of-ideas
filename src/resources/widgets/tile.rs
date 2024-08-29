use bevy::math::FloatExt;
use egui::{Area, NumExt};

use super::*;

pub struct Tile {
    id: String,
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
    side: Side,
    size: f32,
    max_size: f32,
    min_size: f32,
    content_size: egui::Vec2,
    margin_size: egui::Vec2,
    transparent: bool,
    sticky: bool,
    focusable: bool,
}

#[derive(Resource)]
pub struct TileResource {
    tiles: OrderedHashMap<String, Tile>,
    focused: String,
    persistent_tiles: Vec<Tile>,
}

#[derive(Clone, Copy, Default)]
struct TileResponse {
    want_focus: bool,
    want_close: bool,
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
            persistent_tiles: default(),
        }
    }
}

static NEXT_ID: Mutex<u64> = Mutex::new(0);
fn next_id() -> u64 {
    let mut id = NEXT_ID.lock().unwrap();
    *id += 1;
    *id
}

impl Tile {
    pub fn new(side: Side, content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static) -> Self {
        Self {
            id: next_id().to_string(),
            content: Box::new(content),
            side,
            size: 0.0,
            max_size: 0.0,
            min_size: 100.0,
            content_size: default(),
            margin_size: FRAME.total_margin().sum(),
            transparent: false,
            sticky: false,
            focusable: true,
        }
    }
    pub fn set_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
    pub fn push(self, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.focused = self.id.clone();
        tr.tiles.insert(self.id.clone(), self);
    }
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }
    pub fn sticky(mut self) -> Self {
        self.sticky = true;
        self
    }
    pub fn non_focusable(mut self) -> Self {
        self.focusable = false;
        self
    }
    pub fn min_size(mut self, value: f32) -> Self {
        self.min_size = value;
        self
    }
    pub fn max_size(mut self, value: f32) -> Self {
        self.max_size = value;
        self
    }
    fn take_space(&mut self, full: bool, space: &mut egui::Vec2) {
        if self.side.is_x() {
            self.max_size = self.content_size.x.at_most(space.x);
        } else {
            self.max_size = self.content_size.y.at_most(space.y);
        }
        // if full {
        //     *space -= self.content_size + egui::Vec2::splat(MARGIN * 4.0);
        // } else {
        if self.side.is_x() {
            space.x -= self.max_size;
        } else {
            space.y -= self.max_size;
        }
        // }
    }
    fn get_screen_rect(ctx: &egui::Context) -> Rect {
        ctx.data(|r| r.get_temp::<Rect>(screen_rect_id())).unwrap()
    }
    fn set_screen_rect(rect: Rect, ctx: &egui::Context) {
        ctx.data_mut(|w| w.insert_temp(screen_rect_id(), rect));
    }
    fn show(&mut self, focused: bool, ctx: &egui::Context, world: &mut World) -> TileResponse {
        let mut response = TileResponse::default();
        let need_size = if self.side.is_x() {
            self.content_size.x
        } else {
            self.content_size.y
        }
        .at_most(self.max_size)
        .at_least(self.min_size);
        self.size = self.size.lerp(need_size, delta_time(world) * 13.0);

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
        let id = Id::new(&self.id);
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
        if self.focusable && left_mouse_just_released(world) {
            if ctx
                .pointer_interact_pos()
                .is_some_and(|pos| rect.contains(pos))
            {
                response.want_focus = true;
            }
        }

        area.constrain_to(rect.expand2(self.margin_size * 0.5))
            .show(ctx, |ui| {
                frame.show(ui, |ui| {
                    let rect = rect.shrink2(self.margin_size * 0.5);
                    ui.set_clip_rect(rect.expand2(self.margin_size * 0.25));
                    ui.expand_to_include_rect(rect);
                    let ui = &mut ui.child_ui(rect, Layout::top_down(Align::Min), None);
                    if !self.sticky {
                        const CROSS_SIZE: f32 = 13.0;
                        let cross_rect = Rect::from_two_pos(
                            rect.right_top(),
                            rect.right_top() + egui::vec2(-CROSS_SIZE, CROSS_SIZE),
                        );
                        let resp = ui.allocate_rect(cross_rect, Sense::click());
                        if resp.clicked() {
                            response.want_close = true;
                        }
                        let stroke = Stroke {
                            width: 2.0,
                            color: if resp.hovered() { YELLOW } else { VISIBLE_DARK },
                        };
                        ui.painter().line_segment(
                            [cross_rect.left_top(), cross_rect.right_bottom()],
                            stroke,
                        );
                        ui.painter().line_segment(
                            [cross_rect.right_top(), cross_rect.left_bottom()],
                            stroke,
                        );
                    }
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
        response
    }

    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        if tr.tiles.is_empty() {
            return;
        }
        let focused = tr.focused.clone();
        let mut persistent_tiles = mem::take(&mut tr.persistent_tiles);
        let mut tiles = mem::take(&mut tr.tiles);
        Self::set_screen_rect(ctx.available_rect(), ctx);
        let mut available_space = ctx.available_rect().size();
        for (_, tile) in &tiles {
            if tile.side.is_x() {
                available_space.x -= tile.margin_size.x;
            } else {
                available_space.y -= tile.margin_size.y;
            }
        }
        if let Some(focused) = tiles.get_mut(&focused) {
            focused.take_space(true, &mut available_space);
        }

        let mut inds = tiles.keys().cloned().collect_vec();
        let focused_ind = inds
            .iter()
            .position(|k| focused.eq(k))
            .unwrap_or(inds.len() - 1);
        let (left, right) = inds.split_at_mut(focused_ind);
        let mut left_i = left.len() as i32 - 1;
        let mut right_i = 1;
        let l = left.len().max(right.len());
        for _ in 0..=l {
            if let Some(key) = right.get(right_i) {
                if let Some(tile) = tiles.get_mut(key) {
                    tile.take_space(false, &mut available_space);
                }
                right_i += 1;
            }
            if left_i >= 0 {
                if let Some(key) = left.get(left_i as usize) {
                    if let Some(tile) = tiles.get_mut(key) {
                        tile.take_space(false, &mut available_space);
                    }
                    left_i -= 1;
                }
            }
        }

        let mut close = None;
        let mut focus = None;
        for (_, tile) in tiles.iter_mut() {
            let focused = tile.focusable && focused.eq(&tile.id);
            let resp = tile.show(focused, ctx, world);
            if resp.want_close || (focused && !tile.sticky && just_pressed(KeyCode::Escape, world))
            {
                close = Some(tile.id.clone());
            } else if resp.want_focus {
                focus = Some(tile.id.clone());
            }
        }
        if let Some(close) = close {
            tiles.remove(&close);
            if close.eq(&focused) {
                if let Some(last) = tiles.values().last() {
                    focus = Some(last.id.clone());
                }
            }
        }
        for tile in persistent_tiles.iter_mut() {
            tile.show(false, ctx, world);
        }
        let mut tr = world.resource_mut::<TileResource>();
        let new_tiles = mem::take(&mut tr.tiles);
        if !new_tiles.is_empty() {
            tr.focused = new_tiles.values().last().unwrap().id.clone();
            tiles.extend(new_tiles);
        } else if let Some(focus) = focus {
            tr.focused = focus;
        }
        tr.tiles = tiles;
        tr.persistent_tiles = persistent_tiles;
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
            .sticky()
            .push(world),
            GameState::Meta => MetaPlugin::add_tiles(world),
            GameState::Shop => ShopPlugin::add_tiles(world),
            GameState::Battle => BattlePlugin::add_tiles(world),
            GameState::TableView(query) => TableViewPlugin::add_tiles(query, world),
            GameState::GameStart => GameStartPlugin::add_tiles(world),
            GameState::Title => TitlePlugin::add_tiles(world),
            _ => {}
        }
    }
    pub fn setup(world: &mut World) {
        world.resource_mut::<TileResource>().persistent_tiles.push(
            Tile::new(Side::Right, |ui, world| {
                Notification::show_recent(ui, world);
            })
            .max_size(500.0)
            .transparent()
            .sticky(),
        )
    }
}

fn screen_rect_id() -> Id {
    static ID: OnceCell<Id> = OnceCell::new();
    *ID.get_or_init(|| Id::new("available_screen_rect"))
}
