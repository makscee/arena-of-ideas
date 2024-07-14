use super::*;

#[derive(Default)]
pub struct Tile {
    name: &'static str,
    side: Side,
    close_btn: bool,
    title: bool,
    transparent: bool,
    open: bool,
    non_resizable: bool,
    content: Option<Box<dyn FnOnce(&mut Ui, &mut World) + Send + Sync>>,
    child: Option<Box<dyn FnOnce(&egui::Context, &mut World) + Send + Sync>>,
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
    pub fn content(
        mut self,
        content: impl FnOnce(&mut Ui, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.content = Some(Box::new(content));
        self
    }
    pub fn child(
        mut self,
        child: impl FnOnce(&egui::Context, &mut World) + Send + Sync + 'static,
    ) -> Self {
        self.child = Some(Box::new(child));
        self
    }
    pub fn show(self, ctx: &egui::Context, world: &mut World) {
        ctx.add_path(&self.name);
        let content = self.content.unwrap_or(Box::new(|_, _| {}));
        if let Some(child) = self.child {
            child(ctx, world);
        }
        let path = ctx.path();
        let content = |ui: &mut Ui| {
            if self.title {
                self.name.cstr().label(ui);
            }
            ui.vertical_centered_justified(|ui| {
                content(ui, world);
                if self.close_btn && Button::click("Close".into()).gray(ui).ui(ui).clicked() {
                    ui.ctx().flip_path_enabled(&path);
                }
            });
        };
        let mut frame = FRAME;
        if self.transparent {
            frame = frame.fill(Color32::TRANSPARENT);
        }
        let open = self.open || ctx.is_path_enabled(&path);
        match self.side {
            Side::Right => {
                SidePanel::right(Id::new(&path))
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
            Side::Left => {
                SidePanel::left(Id::new(&path))
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
            Side::Top => {
                TopBottomPanel::top(Id::new(&path))
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
            Side::Bottom => {
                TopBottomPanel::bottom(Id::new(&path))
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
        }
        ctx.remove_path();
    }
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(13.0),
    outer_margin: Margin::same(13.0),
    rounding: Rounding::same(13.0),
    shadow: Shadow::NONE,
    fill: BG_LIGHT,
    stroke: Stroke::NONE,
};
