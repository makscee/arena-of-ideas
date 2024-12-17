use bevy_egui::egui::{
    Align, CollapsingHeader, Color32, Frame, Layout, Margin, Rounding, Shadow, Stroke, Ui,
};
use parking_lot::{const_mutex, Mutex};
use ui::*;

use super::*;

pub struct NodeFrame;

static EDITING_WINDOW: Mutex<Option<(Entity, NodeKind)>> = const_mutex(None);

pub fn set_editing_node(entity: Entity, kind: NodeKind) {
    let mut d = EDITING_WINDOW.lock();
    if d.as_ref().is_some_and(|(e, k)| *e == entity && *k == kind) {
        *d = None;
    } else {
        *d = Some((entity, kind));
    }
}
pub fn take_editing_node() -> Option<(Entity, NodeKind)> {
    EDITING_WINDOW.lock().take()
}

impl NodeFrame {
    pub fn show(
        node: &impl Node,
        depth: usize,
        color: Option<Color32>,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui),
    ) {
        let kind = node.kind();
        let name = node
            .get_var(VarName::name)
            .and_then(|v| v.get_string().ok());
        let color = color.unwrap_or(VISIBLE_LIGHT);

        const FRAME: Frame = Frame {
            inner_margin: Margin::same(0.0),
            outer_margin: Margin::same(4.0),
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: BG_DARK,
            stroke: Stroke::NONE,
        };
        let fill = if depth % 2 == 0 { BG_LIGHT } else { BG_DARK };
        ui.style_mut().spacing.item_spacing.y = 0.0;
        FRAME.fill(fill).show(ui, |ui| {
            let mut trounding = Rounding {
                nw: 13.0,
                ne: 0.0,
                sw: 13.0,
                se: 13.0,
            };
            const MARGIN: Margin = Margin {
                left: 4.0,
                right: 4.0,
                top: 0.0,
                bottom: 0.0,
            };
            if let Some(name) = name {
                Frame::none()
                    .stroke(Stroke::new(1.0, color))
                    .rounding(Rounding {
                        nw: 13.0,
                        ne: 13.0,
                        sw: 0.0,
                        se: 0.0,
                    })
                    .inner_margin(MARGIN)
                    .show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| {
                            name.cstr_cs(color, CstrStyle::Bold).label(ui);
                        });
                    });
                trounding.nw = 0.0;
            }
            let visuals = &mut ui.visuals_mut().widgets;
            visuals.inactive.rounding = trounding;
            visuals.hovered.rounding = trounding;
            visuals.active.rounding = trounding;
            visuals.inactive.weak_bg_fill = color;
            visuals.hovered.weak_bg_fill = YELLOW;
            visuals.active.weak_bg_fill = VISIBLE_BRIGHT;
            visuals.inactive.bg_stroke = Stroke::NONE;
            visuals.hovered.weak_bg_fill = YELLOW;
            let rect = CollapsingHeader::new(
                kind.to_string()
                    .cstr_cs(fill, CstrStyle::Small)
                    .widget(1.0, ui),
            )
            .show_background(true)
            .default_open(true)
            .show_unindented(ui, |ui| {
                ui.set_min_width(ui.available_width());
                Frame::none()
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        node.show(None, ui);
                        content(ui);
                    });
            })
            .header_response
            .rect;
            let min_rect = ui.min_rect();
            let rect = rect.with_max_x(min_rect.max.x).with_min_x(rect.max.x);
            ui.reset_style();
            let ui = &mut ui.child_ui(rect, Layout::left_to_right(Align::Center), None);
            if "e".cstr().button(ui).clicked() {
                set_editing_node(node.entity().unwrap(), kind);
            }
        });
    }
}
