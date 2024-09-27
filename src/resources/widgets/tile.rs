use super::*;

pub struct TilePlugin;
impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .init_resource::<TileResource>()
            .init_resource::<ScreenResource>();
    }
}

fn setup(world: &mut World) {
    Tile::new(Side::Right, |ui, world| {
        Notification::show_recent(ui, world);
    })
    .pinned()
    .transparent()
    .no_frame()
    .order(Order::Foreground)
    .push_persistent(world);
}

pub struct Tile {
    id: String,
    side: Side,
    content: Box<dyn Fn(&mut Ui, &mut World) + Send + Sync>,
    order: Order,
    actual_space: egui::Vec2,
    allocated_space: egui::Vec2,
    content_space: egui::Vec2,
    margin_space: egui::Vec2,
    min_space: egui::Vec2,
    pinned: bool,
    focusable: bool,
    transparent: bool,
    framed: bool,
    open: bool,
}

#[derive(Resource, Default)]
struct TileResource {
    tiles: OrderedHashMap<String, Tile>,
    focused: String,
    persistent_overlay: Vec<Tile>,
    content: Option<Tile>,
}

#[derive(Resource)]
struct ScreenResource {
    screen_rect: Rect,
    screen_space: egui::Vec2,
}

const MARGIN: f32 = 7.0;
const FRAME: Frame = Frame {
    inner_margin: Margin::same(MARGIN),
    outer_margin: Margin::same(MARGIN),
    rounding: Rounding::same(13.0),
    shadow: Shadow::NONE,
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
            screen_rect: Rect::NOTHING,
        }
    }
}

impl TilePlugin {
    pub fn show_all(ctx: &egui::Context, world: &mut World) {
        Self::reset(world);
        let mut tr = mem::take(world.resource_mut::<TileResource>().as_mut());
        let mut sr = world.remove_resource::<ScreenResource>().unwrap();

        let focused = tr.focused.clone();
        for tile in tr.tiles.values_mut() {
            tile.allocate_margin_space(false, &mut sr);
        }
        if let Some(focused) = tr.tiles.get_mut(&focused) {
            focused.allocate_content_space(false, &mut sr);
        }
        if let Some(content) = &mut tr.content {
            content.allocate_margin_space(true, &mut sr);
            content.allocate_content_space(true, &mut sr);
        }
        for tile in tr.tiles.values_mut().rev() {
            if focused.eq(&tile.id) {
                continue;
            }
            tile.allocate_content_space(false, &mut sr);
        }
        let mut remove = None;
        let mut new_focus = None;
        for tile in tr.tiles.values_mut() {
            let focused = focused.eq(&tile.id);
            if tile.show(focused, &mut sr, ctx, world) {
                new_focus = Some(tile.id.clone());
            }
            if !tile.open && tile.actual_space.length() < 1.0 {
                remove = Some(tile.id.clone());
            }
        }
        let content_space = sr.screen_rect;
        if let Some(content) = &mut tr.content {
            content.show(false, &mut sr, ctx, world);
        }
        sr.screen_rect = content_space;
        sr.screen_space = content_space.size();
        for tile in &mut tr.persistent_overlay {
            tile.allocate_margin_space(false, &mut sr);
            tile.allocate_content_space(false, &mut sr);
            tile.show(false, &mut sr, ctx, world);
        }
        if let Some(remove) = remove {
            tr.tiles.remove(&remove);
        }
        if let Some(focus) = new_focus {
            tr.focused = focus;
        }
        let mut new_tr = world.resource_mut::<TileResource>();
        tr.tiles.extend(new_tr.tiles.drain());
        if !new_tr.focused.is_empty() {
            tr.focused = mem::take(&mut new_tr.focused);
        }
        *new_tr = tr;

        world.insert_resource(sr);
    }
    fn reset(world: &mut World) {
        let ctx = &egui_context(world).unwrap();
        let mut sr = world.resource_mut::<ScreenResource>();
        sr.screen_rect = ctx.available_rect();
        sr.screen_space = sr.screen_rect.size();
        let mut tr = world.resource_mut::<TileResource>();
        for tile in tr.tiles.values_mut() {
            tile.allocated_space = default();
        }
        for tile in &mut tr.persistent_overlay {
            tile.allocated_space = default();
        }
        if let Some(content) = &mut tr.content {
            content.allocated_space = default();
        }
    }
    fn clear(world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.tiles.clear();
        tr.content = None;
    }

    pub fn add_team(gid: u64, world: &mut World) {
        Tile::new(Side::Right, move |ui, world| {
            gid.get_team().show(ui, world);
        })
        .push(world)
    }
    pub fn add_user(gid: u64, world: &mut World) {
        Tile::new(Side::Right, move |ui, world| {
            gid.get_user().show(ui, world);
        })
        .push(world)
    }
    pub fn add_fused_unit(unit: FusedUnit, world: &mut World) {
        Tile::new(Side::Right, move |ui, world| {
            unit.show(ui, world);
        })
        .push(world)
    }
    pub fn change_state(to: GameState, world: &mut World) {
        Self::clear(world);
        match to {
            GameState::Inbox => Tile::new(Side::Left, |ui, world| {
                Notification::show_all_table(ui, world)
            })
            .pinned()
            .push(world),
            GameState::MetaShop
            | GameState::MetaAuction
            | GameState::MetaHeroes
            | GameState::MetaHeroShards
            | GameState::MetaLootboxes => MetaPlugin::add_tiles(world),
            GameState::Shop => ShopPlugin::add_tiles(world),
            GameState::Battle => BattlePlugin::add_tiles(world),
            GameState::TableView(query) => TableViewPlugin::add_tiles(query, world),
            GameState::GameStart => GameStartPlugin::add_tiles(world),
            GameState::Title => TitlePlugin::add_tiles(world),
            GameState::Teams | GameState::TeamEditor => TeamPlugin::add_tiles(to, world),
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
            order: Order::Middle,
            actual_space: default(),
            allocated_space: default(),
            content_space: default(),
            margin_space: FRAME.total_margin().sum(),
            open: true,
            pinned: false,
            focusable: true,
            transparent: false,
            framed: true,
            min_space: default(),
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
    pub fn set_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
    #[must_use]
    pub fn min_space(mut self, value: egui::Vec2) -> Self {
        self.min_space = value;
        self
    }
    #[must_use]
    pub fn order(mut self, value: Order) -> Self {
        self.order = value;
        self
    }
    pub fn push(self, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        if self.focusable {
            tr.focused = self.id.clone();
        }
        tr.tiles.insert(self.id.clone(), self);
    }
    pub fn push_persistent(self, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.persistent_overlay.push(self);
    }
    pub fn push_as_content(self, world: &mut World) {
        let mut tr = world.resource_mut::<TileResource>();
        tr.content = Some(self);
    }

    fn allocate_space(&mut self, mut space: egui::Vec2, full: bool, sr: &mut ScreenResource) {
        if !self.open {
            return;
        }
        if self.side.is_x() && !full {
            space.y = 0.0;
        }
        if self.side.is_y() && !full {
            space.x = 0.0;
        }
        self.allocated_space += space;
        sr.screen_space -= space;
    }
    fn allocate_margin_space(&mut self, full: bool, sr: &mut ScreenResource) {
        let space = self.margin_space.at_most(sr.screen_space);
        self.allocate_space(space, full, sr);
    }
    fn allocate_content_space(&mut self, full: bool, sr: &mut ScreenResource) {
        let space = self.content_space.at_most(sr.screen_space);
        self.allocate_space(space, full, sr);
    }
    fn show(
        &mut self,
        focused: bool,
        sr: &mut ScreenResource,
        ctx: &egui::Context,
        world: &mut World,
    ) -> bool {
        let mut response = false;
        self.actual_space += (self.allocated_space - self.actual_space)
            * delta_time(world).at_most(1.0 / 60.0)
            * 13.0;
        let id = Id::new(&self.id);
        let (mut area, rect) = match self.side {
            Side::Right => (
                Area::new(id)
                    .pivot(Align2::RIGHT_TOP)
                    .fixed_pos(sr.screen_rect.right_top()),
                sr.screen_rect
                    .with_min_x(sr.screen_rect.max.x - self.actual_space.x),
            ),
            Side::Left => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(sr.screen_rect.left_top()),
                sr.screen_rect
                    .with_max_x(sr.screen_rect.min.x + self.actual_space.x),
            ),
            Side::Top => (
                Area::new(id)
                    .pivot(Align2::LEFT_TOP)
                    .fixed_pos(sr.screen_rect.left_top()),
                sr.screen_rect
                    .with_max_y(sr.screen_rect.min.y + self.actual_space.y),
            ),
            Side::Bottom => (
                Area::new(id)
                    .pivot(Align2::LEFT_BOTTOM)
                    .fixed_pos(sr.screen_rect.left_bottom()),
                sr.screen_rect
                    .with_min_y(sr.screen_rect.max.y - self.actual_space.y),
            ),
        };
        area = area.order(self.order);
        match self.side {
            Side::Right => sr.screen_rect.max.x -= self.actual_space.x,
            Side::Left => sr.screen_rect.min.x += self.actual_space.x,
            Side::Top => sr.screen_rect.min.y += self.actual_space.y,
            Side::Bottom => sr.screen_rect.max.y -= self.actual_space.y,
        }
        if self.focusable && left_mouse_just_released(world) {
            if ctx
                .pointer_interact_pos()
                .is_some_and(|pos| rect.contains(pos))
            {
                response = true;
            }
        }
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
            frame = frame.fill(Color32::TRANSPARENT);
        }

        area.constrain_to(rect).show(ctx, |ui| {
            frame.show(ui, |ui| {
                let content_rect = rect.shrink2(frame.total_margin().sum() * 0.5);
                ui.expand_to_include_rect(content_rect);
                ui.set_max_size(content_rect.size().at_least(egui::vec2(0.1, 0.1)));
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
                self.content_space = ui.min_size().at_least(self.min_space);
            });
        });
        response
    }
}
