use lerp::Lerp;

use super::*;

pub struct TilePlugin;
impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileResource>()
            .init_resource::<ScreenResource>();
    }
}

pub struct Tile {
    id: String,
    side: Side,
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
    content_space: egui::Vec2,
    allocated_space: egui::Vec2,
    min_space: egui::Vec2,
    pinned: bool,
    focusable: bool,
    transparent: bool,
    framed: bool,
    open: bool,
    no_margin: bool,
    no_expand: bool,
    keep: bool,
    extension: f32,
    stretch_mode: StretchMode,
}

#[derive(Default, PartialEq)]
enum StretchMode {
    #[default]
    Floating,
    Min,
    Max,
    Part(f32),
}

#[derive(Default)]
enum TileResponse {
    #[default]
    None,
    WantFocus,
}

#[derive(Resource, Default)]
struct TileResource {
    tiles: IndexMap<String, Tile>,
    focused: String,
    new_tiles: Vec<Tile>,
    close_tiles: Vec<String>,
    close_not_kept: bool,
}
fn rm(world: &mut World) -> Mut<TileResource> {
    world.resource_mut::<TileResource>()
}
fn r(world: &World) -> &TileResource {
    world.resource::<TileResource>()
}

#[derive(Resource)]
struct ScreenResource {
    screen_space: egui::Vec2,
    screen_space_initial: egui::Vec2,
    screen_mood: egui::Vec2,
    screen_rect: Rect,
}

const MARGIN: f32 = 7.0;
const FRAME: Frame = Frame {
    inner_margin: Margin::same(MARGIN),
    outer_margin: Margin::same(MARGIN),
    rounding: Rounding::same(13.0),
    shadow: SHADOW,
    fill: BG_DARK,
    stroke: Stroke {
        width: 1.0,
        color: BG_LIGHT,
    },
};

static NEXT_ID: Mutex<u64> = Mutex::new(0);
fn next_id() -> u64 {
    let mut id = NEXT_ID.lock().unwrap();
    *id += 1;
    *id
}

impl Default for ScreenResource {
    fn default() -> Self {
        Self {
            screen_space: default(),
            screen_space_initial: default(),
            screen_mood: egui::vec2(1.0, 1.0),
            screen_rect: Rect::NOTHING,
        }
    }
}

impl TilePlugin {
    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        Self::reset(world);
        let dt = delta_time(world) * 13.0;
        let mut sr = world.remove_resource::<ScreenResource>().unwrap();
        let mood = sr.screen_mood;
        let mut tr = rm(world);

        let focused = tr.focused.clone();
        if let Some(focused) = tr.tiles.get_mut(&focused) {
            focused.allocate_space(egui::vec2(1.0, 1.0), &mut sr, dt);
        }
        for (_, tile) in tr
            .tiles
            .iter_mut()
            .filter(|(id, t)| t.stretch_mode == StretchMode::Min && !focused.eq(*id))
        {
            tile.allocate_space(mood, &mut sr, dt);
        }
        for (_, tile) in tr.tiles.iter_mut().filter(|(id, t)| {
            (t.stretch_mode == StretchMode::Floating
                || matches!(t.stretch_mode, StretchMode::Part(..)))
                && !focused.eq(*id)
        }) {
            tile.allocate_space(mood, &mut sr, dt);
        }
        if let Some((_, tile)) = tr
            .tiles
            .iter_mut()
            .find(|(_, t)| t.stretch_mode == StretchMode::Max)
        {
            tile.allocate_space(mood, &mut sr, dt);
        }
        let mut tiles_len = tr.tiles.len();

        let mut i = 0;
        while i < tiles_len {
            let Some((_, mut tile)) = rm(world).tiles.swap_remove_index(i) else {
                break;
            };
            let tile_focused = focused.eq(&tile.id);
            match tile.show(tile_focused, &mut sr, ctx, world) {
                TileResponse::None => {}
                TileResponse::WantFocus => {
                    let mut rm = rm(world);
                    if focused.eq(&rm.focused) {
                        rm.focused = tile.id.clone();
                    }
                }
            }
            let tiles = &mut rm(world).tiles;
            let mut removed = false;
            if tile.open || tile.extension > 0.01 {
                tiles.insert(tile.id.clone(), tile);
                tiles.swap_indices(i, tiles_len - 1);
            } else {
                if let Some((key, tile)) = tiles.shift_remove_index(i) {
                    tiles.insert(key, tile);
                    tiles_len -= 1;
                    removed = true;
                }
            }
            if !removed {
                i += 1;
            }
        }
        let mut tr = rm(world);
        if mem::take(&mut tr.close_not_kept) {
            let tiles = tr
                .tiles
                .iter()
                .filter_map(|(k, v)| if v.keep { None } else { Some(k.into()) })
                .collect_vec();
            tr.close_tiles.extend(tiles);
        }
        for id in mem::take(&mut tr.close_tiles) {
            tr.tiles.shift_remove(&id);
        }
        for tile in mem::take(&mut tr.new_tiles) {
            if tile.focusable {
                tr.focused = tile.id.clone();
            }
            if let Some(tile) = tr.tiles.get_mut(&tile.id) {
                tile.open = false;
            } else {
                tr.tiles.insert(tile.id.clone(), tile);
            }
        }
        sr.screen_mood += egui::vec2(
            sr.screen_space.x / sr.screen_space_initial.x,
            sr.screen_space.y / sr.screen_space_initial.y,
        ) * dt;
        sr.screen_mood = sr
            .screen_mood
            .clamp(egui::vec2(-1.0, -1.0), egui::vec2(1.0, 1.0));

        world.insert_resource(sr);
        if just_pressed(KeyCode::Escape, world) {
            let mut rm = rm(world);
            let focused = rm.focused.clone();
            if let Some(tile) = rm.tiles.get_mut(&focused).filter(|t| !t.pinned) {
                tile.open = false;
                rm.focused = default()
            } else {
                for (_, tile) in rm.tiles.iter_mut().rev() {
                    if tile.pinned || !tile.open {
                        continue;
                    }
                    tile.open = false;
                    break;
                }
            }
        }
    }
    fn reset(world: &mut World) {
        let ctx = &egui_context(world).unwrap();
        let mut sr = world.resource_mut::<ScreenResource>();
        sr.screen_rect = ctx.available_rect();
        sr.screen_space = sr.screen_rect.size();
        sr.screen_space_initial = sr.screen_space;
    }
    pub fn clear(world: &mut World) {
        rm(world).close_not_kept = true;
    }
    fn clear_all(world: &mut World) {
        rm(world).tiles.clear();
    }

    pub fn add_team(gid: u64, world: &mut World) {
        Tile::new(Side::Right, move |ui, world| {
            gid.get_team().show(ui, world);
        })
        .with_id(format!("team_{gid}"))
        .push(world)
    }
    pub fn add_user(gid: u64, world: &mut World) {
        Tile::new(Side::Right, move |ui, world| {
            gid.get_player().show(ui, world);
        })
        .with_id(format!("user_{gid}"))
        .push(world)
    }
    pub fn add_fused_unit(unit: FusedUnit, world: &mut World) {
        let gid = unit.id;
        Tile::new(Side::Right, move |ui, world| {
            unit.show(ui, world);
        })
        .with_id(format!("unit_{gid}"))
        .push(world)
    }
    pub fn is_open(id: &str, world: &World) -> bool {
        r(world).tiles.contains_key(id)
    }
    pub fn close(id: &str, world: &mut World) {
        rm(world).close_tiles.push(id.into());
    }
    pub fn change_state(to: GameState, world: &mut World) {
        Self::clear_all(world);
        match to {
            GameState::Inbox => InboxPlugin::add_tiles(world),
            GameState::Meta => MetaPlugin::add_tiles(world),
            GameState::Shop => ShopPlugin::add_tiles(world),
            GameState::Battle => BattlePlugin::add_tiles(world),
            GameState::GameStart => GameStartPlugin::add_tiles(world),
            GameState::Title => TitlePlugin::add_tiles(world),
            GameState::Editor => EditorPlugin::add_tiles(world),
            GameState::Quests => QuestPlugin::add_tiles(world),
            GameState::Stats => StatsPlugin::add_tiles(world),
            GameState::Incubator => IncubatorPlugin::add_tiles(world),
            GameState::Players => PlayersPlugin::add_tiles(world),
            _ => {}
        }
    }
}

impl Tile {
    #[must_use]
    pub fn new(side: Side, content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static) -> Self {
        Self {
            id: next_id().to_string(),
            content: Box::new(content),
            side,
            content_space: default(),
            open: true,
            pinned: false,
            focusable: true,
            transparent: false,
            framed: true,
            min_space: default(),
            no_margin: false,
            no_expand: false,
            keep: false,
            extension: 0.0,
            stretch_mode: default(),
            allocated_space: default(),
        }
    }
    #[must_use]
    pub fn non_focusable(mut self) -> Self {
        self.focusable = false;
        self
    }
    #[must_use]
    pub fn pinned(mut self) -> Self {
        self.pinned = true;
        self
    }
    #[must_use]
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }
    #[must_use]
    pub fn no_frame(mut self) -> Self {
        self.framed = false;
        self
    }
    #[must_use]
    pub fn no_margin(mut self) -> Self {
        self.no_margin = true;
        self
    }
    #[must_use]
    pub fn no_expand(mut self) -> Self {
        self.no_expand = true;
        self
    }
    #[must_use]
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
    #[must_use]
    pub fn min_space(mut self, value: egui::Vec2) -> Self {
        self.min_space = value;
        self
    }
    #[must_use]
    pub fn stretch_max(mut self) -> Self {
        self.stretch_mode = StretchMode::Max;
        self.focusable = false;
        self
    }
    #[must_use]
    pub fn stretch_min(mut self) -> Self {
        self.stretch_mode = StretchMode::Min;
        self.focusable = false;
        self
    }
    #[must_use]
    pub fn stretch_part(mut self, v: f32) -> Self {
        self.stretch_mode = StretchMode::Part(v);
        self.focusable = false;
        self
    }
    #[must_use]
    pub fn keep(mut self) -> Self {
        self.keep = true;
        self
    }
    pub fn push(self, world: &mut World) {
        let mut tr = rm(world);
        tr.new_tiles.push(self);
    }
    pub fn push_front(self, world: &mut World) {
        let mut tr = rm(world);
        tr.new_tiles.insert(0, self);
    }

    fn allocate_space(&mut self, mood: egui::Vec2, sr: &mut ScreenResource, dt: f32) {
        if !self.open {
            self.extension.lerp_to(0.0, dt);
        }
        match self.stretch_mode {
            StretchMode::Floating => {
                if self.open {
                    if self.side.is_x() {
                        self.extension.lerp_to(mood.x, dt);
                    } else {
                        self.extension.lerp_to(mood.y, dt);
                    }
                }
            }
            StretchMode::Min | StretchMode::Part(..) => self.extension.lerp_to(1.0, dt),
            StretchMode::Max => {}
        }
        self.allocated_space = match self.stretch_mode {
            StretchMode::Min | StretchMode::Floating => {
                if self.side.is_x() {
                    egui::vec2(self.content_space.x * self.extension, 0.0)
                } else {
                    egui::vec2(0.0, self.content_space.y * self.extension)
                }
            }
            StretchMode::Max => sr.screen_space,
            StretchMode::Part(v) => {
                if self.side.is_x() {
                    egui::vec2(sr.screen_space.x * v * self.extension, 0.0)
                } else {
                    egui::vec2(0.0, sr.screen_space.y * v * self.extension)
                }
            }
        };
        sr.screen_space -= self.allocated_space;
    }
    fn show(
        &mut self,
        focused: bool,
        sr: &mut ScreenResource,
        ctx: &egui::Context,
        world: &mut World,
    ) -> TileResponse {
        let mut response = TileResponse::None;
        let id = Id::new(&self.id);

        let (area, rect) = match self.side {
            Side::Right => (
                Area::new(id)
                    .pivot(Align2::RIGHT_TOP)
                    .fixed_pos(sr.screen_rect.right_top()),
                sr.screen_rect
                    .with_min_x(sr.screen_rect.max.x - self.allocated_space.x),
            ),
            Side::Left => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(sr.screen_rect.left_top()),
                sr.screen_rect
                    .with_max_x(sr.screen_rect.min.x + self.allocated_space.x),
            ),
            Side::Top => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(sr.screen_rect.left_top()),
                sr.screen_rect
                    .with_max_y(sr.screen_rect.min.y + self.allocated_space.y),
            ),
            Side::Bottom => (
                Area::new(id)
                    .pivot(Align2::LEFT_BOTTOM)
                    .fixed_pos(sr.screen_rect.left_bottom()),
                sr.screen_rect
                    .with_min_y(sr.screen_rect.max.y - self.allocated_space.y),
            ),
        };

        match self.side {
            Side::Right => sr.screen_rect.max.x -= self.allocated_space.x,
            Side::Left => sr.screen_rect.min.x += self.allocated_space.x,
            Side::Top => sr.screen_rect.min.y += self.allocated_space.y,
            Side::Bottom => sr.screen_rect.max.y -= self.allocated_space.y,
        }
        // if !self.pinned && right_mouse_just_released(world) {
        //     if ctx
        //         .pointer_interact_pos()
        //         .is_some_and(|pos| rect.contains(pos))
        //     {
        //         self.open = false;
        //     }
        // }
        let mut frame = if focused {
            FRAME.stroke(Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            })
        } else if !self.framed {
            FRAME.stroke(Stroke::NONE)
        } else {
            FRAME
        };
        if self.transparent {
            frame.fill = Color32::TRANSPARENT;
            frame.shadow.color = frame.shadow.color.gamma_multiply(0.3);
        } else if focused {
            frame.shadow.color = frame.shadow.color.lerp_to_gamma(Color32::WHITE, 0.07);
        }
        if self.no_margin {
            frame.inner_margin = default();
            frame.outer_margin = default();
        }
        if !rect.area().is_finite() {
            return response;
        }
        let area_response = area.sense(Sense::click()).show(ctx, |ui| {
            let mut content_rect = rect.shrink2(frame.total_margin().sum() * 0.5);
            if self.no_expand {
                match self.side {
                    Side::Right | Side::Left => {
                        content_rect
                            .set_height(self.content_space.y - frame.total_margin().sum().y);
                    }
                    Side::Top | Side::Bottom => {
                        content_rect.set_width(self.content_space.x - frame.total_margin().sum().x);
                    }
                }
            }
            ui.expand_to_include_rect(content_rect);

            ui.painter()
                .add(frame.paint(content_rect.expand2(frame.inner_margin.sum() * 0.5)));
            let ui = &mut ui.child_ui(content_rect, *ui.layout(), None);
            ui.set_clip_rect(content_rect.expand(2.0));
            if !self.pinned {
                const CROSS_SIZE: f32 = 13.0;
                let cross_rect = Rect::from_two_pos(
                    content_rect.right_top(),
                    content_rect.right_top() + egui::vec2(-CROSS_SIZE, CROSS_SIZE),
                );
                let resp = ui.allocate_rect(cross_rect, Sense::click());
                if resp.clicked() {
                    self.open = false;
                }
                let stroke = Stroke {
                    width: 2.0,
                    color: if resp.hovered() { YELLOW } else { VISIBLE_DARK },
                };
                ui.painter()
                    .line_segment([cross_rect.left_top(), cross_rect.right_bottom()], stroke);
                ui.painter()
                    .line_segment([cross_rect.right_top(), cross_rect.left_bottom()], stroke);
            }
            (self.content)(ui, world);
            self.content_space =
                ui.min_size().at_least(self.min_space) + frame.total_margin().sum();
        });

        if self.focusable && area_response.response.clicked() {
            response = TileResponse::WantFocus;
        }
        response
    }
}
