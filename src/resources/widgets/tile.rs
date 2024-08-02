use super::*;

pub struct Tile {
    name: &'static str,
    id: Option<Id>,
    side: Side,
    close_btn: bool,
    title: bool,
    transparent: bool,
    open: bool,
    non_resizable: bool,
    default_size: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            name: default(),
            id: default(),
            side: default(),
            close_btn: default(),
            title: default(),
            transparent: default(),
            open: default(),
            non_resizable: default(),
            default_size: 300.0,
        }
    }
}

impl Tile {
    pub fn right(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Right,
            ..default()
        }
    }
    pub fn left(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Left,
            ..default()
        }
    }
    pub fn top(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Top,
            ..default()
        }
    }
    pub fn bottom(name: &'static str) -> Self {
        Self {
            name,
            side: Side::Bottom,
            ..default()
        }
    }
    pub fn with_id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }
    pub fn title(mut self) -> Self {
        self.title = true;
        self
    }
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }
    pub fn close_btn(mut self) -> Self {
        self.close_btn = true;
        self
    }
    pub fn open(mut self) -> Self {
        self.open = true;
        self
    }
    pub fn non_resizable(mut self) -> Self {
        self.non_resizable = true;
        self
    }
    pub fn default_size(mut self, value: f32) -> Self {
        self.default_size = value;
        self
    }
    pub fn show(self, ctx: &egui::Context, content: impl FnOnce(&mut Ui)) {
        self.show_data(&mut Some(()), ctx, |_, ui| content(ui));
    }
    pub fn show_data<T>(
        self,
        data: &mut Option<T>,
        ctx: &egui::Context,
        content: impl FnOnce(&mut T, &mut Ui),
    ) {
        let mut frame = FRAME;
        if self.transparent {
            frame = frame.fill(Color32::TRANSPARENT);
        }
        let open = data.is_some();
        let id = Id::new(self.name).with(self.id);
        let content = |ui: &mut Ui| {
            if self.title {
                self.name.cstr().label(ui);
            }
            ui.vertical_centered_justified(|ui| {
                if let Some(data) = data {
                    content(data, ui);
                }
                if self.close_btn && Button::click("Close".into()).gray(ui).ui(ui).clicked() {
                    data.take();
                }
            });
        };
        match self.side {
            Side::Right => SidePanel::right(id)
                .frame(frame)
                .default_width(self.default_size)
                .resizable(!self.non_resizable)
                .show_separator_line(false)
                .show_animated(ctx, open, content),
            Side::Left => SidePanel::left(id)
                .frame(frame)
                .default_width(self.default_size)
                .resizable(!self.non_resizable)
                .show_separator_line(false)
                .show_animated(ctx, open, content),
            Side::Top => TopBottomPanel::top(id)
                .frame(frame)
                .default_height(self.default_size)
                .resizable(!self.non_resizable)
                .show_separator_line(false)
                .show_animated(ctx, open, content),
            Side::Bottom => TopBottomPanel::bottom(id)
                .frame(frame)
                .default_height(self.default_size)
                .resizable(!self.non_resizable)
                .show_separator_line(false)
                .show_animated(ctx, open, content),
        };
    }

    pub fn add_team(gid: GID, ctx: &egui::Context) {
        ctx.data_mut(|w| {
            let m: &mut TileMap = w.get_temp_mut_or_default(tile_map_id());
            m.insert(gid, TileContent::Team(TTeam::filter_by_id(gid)));
        });
    }
    pub fn add_user(gid: GID, ctx: &egui::Context) {
        ctx.data_mut(|w| {
            let m: &mut TileMap = w.get_temp_mut_or_default(tile_map_id());
            m.insert(gid, TileContent::User(TUser::filter_by_id(gid)));
        });
    }
    pub fn add_fused_unit(unit: FusedUnit, ctx: &egui::Context) {
        ctx.data_mut(|w| {
            let m: &mut TileMap = w.get_temp_mut_or_default(tile_map_id());
            m.insert(unit.id, TileContent::FusedUnit(Some(unit)));
        });
    }

    pub fn show_all_tiles(ctx: &egui::Context, world: &mut World) {
        let id = tile_map_id();
        let mut m: TileMap = ctx.data(|r| r.get_temp(id)).unwrap_or_default();
        for (gid, tile) in &mut m {
            match tile {
                TileContent::Team(t) => Tile::left("Team")
                    .with_id(Id::new(*gid))
                    .close_btn()
                    .show_data(t, ctx, |t, ui| t.show(ui, world)),
                TileContent::FusedUnit(u) => Tile::left("Unit")
                    .with_id(Id::new(*gid))
                    .close_btn()
                    .show_data(u, ctx, |t, ui| t.show(ui, world)),
                TileContent::User(u) => Tile::left("User")
                    .with_id(Id::new(*gid))
                    .close_btn()
                    .show_data(u, ctx, |t, ui| t.show(ui, world)),
            };
        }
        ctx.data_mut(|w| {
            let t = w.get_temp_mut_or_default::<TileMap>(id);
            for (k, v) in m {
                t.insert(k, v);
            }
        });
    }
}

fn tile_map_id() -> Id {
    static TILES_DATA: OnceCell<Id> = OnceCell::new();
    *TILES_DATA.get_or_init(|| Id::new("tiles_data"))
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(13.0),
    outer_margin: Margin::same(13.0),
    rounding: Rounding::same(13.0),
    shadow: Shadow::NONE,
    fill: BG_DARK,
    stroke: Stroke::NONE,
};

type TileMap = OrderedHashMap<GID, TileContent>;

#[derive(Clone, Debug)]
enum TileContent {
    Team(Option<TTeam>),
    FusedUnit(Option<FusedUnit>),
    User(Option<TUser>),
}
