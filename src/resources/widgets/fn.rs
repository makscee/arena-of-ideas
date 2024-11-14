use egui::ScrollArea;

use super::*;

pub fn br(ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.max_rect();
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            },
        );
    });
}
pub fn space(ui: &mut Ui) {
    ui.add_space(13.0);
}
pub fn center_window(name: &str, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui)) {
    Window::new(name)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(ctx.screen_rect().center())
        .title_bar(false)
        .order(Order::Foreground)
        .default_width(300.0)
        .resizable([false, false])
        .show(ctx, |ui| {
            ui.set_max_height(ui.ctx().screen_rect().height() * 0.9);
            ScrollArea::vertical().show(ui, add_contents);
        });
}
pub fn popup(name: &str, ctx: &egui::Context, add_contents: impl FnOnce(&mut Ui)) {
    let rect = ctx.screen_rect();
    Area::new(Id::new("black_bg"))
        .constrain_to(rect)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .sense(Sense::click())
        .show(ctx, |ui| {
            ui.expand_to_include_rect(rect);
            ui.painter_at(rect)
                .rect_filled(rect, Rounding::ZERO, Color32::from_black_alpha(200));
        });
    center_window(name, ctx, add_contents);
}
pub fn text_dots_text(text1: Cstr, text2: Cstr, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let rect = ui.available_rect_before_wrap();
        let left_width = text1.label(ui).rect.width();
        let left = rect.left() + left_width + 3.0;
        let right_width = ui
            .with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                text2.label(ui);
            })
            .response
            .rect
            .width();
        let right = rect.right() - 3.0 - right_width;
        let bottom = rect.bottom() - 6.0;
        let line = egui::Shape::dotted_line(
            &[[left, bottom].into(), [right, bottom].into()],
            VISIBLE_LIGHT,
            12.0,
            0.5,
        );
        ui.expand_to_include_x(rect.left() + left_width + right_width + 30.0);
        ui.painter().add(line);
    });
}
pub fn title(text: &str, ui: &mut Ui) {
    text.cstr_cs(VISIBLE_DARK, CstrStyle::Heading2).label(ui);
    br(ui);
}

pub fn cursor_window(ctx: &egui::Context, content: impl FnOnce(&mut Ui)) {
    let mut pos = ctx.pointer_latest_pos().unwrap_or_default();
    const WIDTH: f32 = 350.0;
    let pivot = if pos.x > ctx.screen_rect().right() - WIDTH {
        pos.x -= 10.0;
        Align2::RIGHT_CENTER
    } else {
        pos.x += 10.0;
        Align2::LEFT_CENTER
    };
    Window::new("cursor_window")
        .title_bar(false)
        .frame(Frame::none())
        .max_width(WIDTH)
        .pivot(pivot)
        .fixed_pos(pos)
        .resizable(false)
        .interactable(false)
        .order(Order::Tooltip)
        .show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                content(ui);
            });
        });
}
pub fn status_selector(status: &mut String, ui: &mut Ui) -> bool {
    Selector::new("status").ui_iter(status, game_assets().statuses.keys(), ui)
}
pub fn ability_selector(ability: &mut String, ui: &mut Ui) -> bool {
    Selector::new("ability").ui_iter(ability, game_assets().abilities.keys(), ui)
}
pub fn summon_selector(summon: &mut String, ui: &mut Ui) -> bool {
    Selector::new("summon").ui_iter(summon, game_assets().summons.keys(), ui)
}
pub fn vfx_selector(vfx: &mut String, ui: &mut Ui) -> bool {
    Selector::new("vfx").ui_iter(vfx, game_assets().vfxs.keys(), ui)
}
pub fn var_selector(var: &mut VarName, ui: &mut Ui) -> bool {
    Selector::new("var").ui_enum(var, ui)
}
pub fn show_collapsing_node<T: ShowEditor>(
    name: &str,
    node: &mut T,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    ui.collapsing(name, |ui| {
        node.show_node(name, context, world, ui);
    });
}
pub fn show_list_node<E: ShowEditor>(
    l: &mut Vec<Box<E>>,
    context: &Context,
    ui: &mut Ui,
    world: &mut World,
) {
    let mut to_remove = None;
    for (i, e) in l.into_iter().enumerate() {
        ui.push_id(i, |ui| {
            ui.horizontal(|ui| {
                if Button::click("-").red(ui).ui(ui).clicked() {
                    to_remove = Some(i);
                }
                e.show_node("", context, world, ui);
            });
        });
    }
    if let Some(i) = to_remove {
        l.remove(i);
    }
}
pub fn season_switcher(value: &mut u32, ui: &mut Ui) -> bool {
    EnumSwitcher::new()
        .prefix("Season ".cstr())
        .show_iter(value, 0..=global_settings().season, ui)
}
pub fn game_mode_switcher(value: &mut GameMode, ui: &mut Ui) -> bool {
    EnumSwitcher::new()
        .style(CstrStyle::Bold)
        .columns()
        .show_iter(
            value,
            [
                GameMode::ArenaNormal,
                GameMode::ArenaRanked,
                GameMode::ArenaConst,
            ],
            ui,
        )
}
