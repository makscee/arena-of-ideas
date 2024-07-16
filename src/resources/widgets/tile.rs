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
        let id = Id::new(self.name);
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
            Side::Right => {
                SidePanel::right(id)
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
            Side::Left => {
                SidePanel::left(id)
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
            Side::Top => {
                TopBottomPanel::top(id)
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
            Side::Bottom => {
                TopBottomPanel::bottom(id)
                    .frame(frame)
                    .resizable(!self.non_resizable)
                    .show_separator_line(false)
                    .show_animated(ctx, open, content);
            }
        }
    }
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(13.0),
    outer_margin: Margin::same(13.0),
    rounding: Rounding::same(13.0),
    shadow: Shadow::NONE,
    fill: BG_DARK,
    stroke: Stroke::NONE,
};
