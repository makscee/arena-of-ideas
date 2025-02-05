use super::*;

#[derive(Default)]
pub struct IconMenu {
    icons: Vec<Icon>,
}

struct Icon {
    name: char,
    color: Color32,
    on_click: fn(&mut World),
    indicator: Option<fn(&World) -> bool>,
    hint: &'static str,
}

impl IconMenu {
    pub fn show(self, ui: &mut Ui, world: &mut World) {
        const TICK: f32 = 3.0;
        let blink = (gt().play_head() / TICK).fract() * 3.0;
        let ticked = gt().ticked(TICK, 0.0);
        for i in self.icons {
            let id = Id::new(i.name);
            if ticked {
                if let Some(indicator) = i.indicator {
                    set_ctx_bool_id(ui.ctx(), id, indicator(world));
                }
            }
            let resp = Button::new(i.name.to_string().cstr_cs(i.color, CstrStyle::Bold))
                .ui(ui)
                .on_hover_text(i.hint);
            if resp.clicked() {
                (i.on_click)(world);
            }
            let rect = resp.rect;
            let show_indicator = get_ctx_bool_id(ui.ctx(), id).unwrap_or_default();
            if show_indicator {
                ui.painter().rect_stroke(
                    rect.expand(blink * 4.0),
                    Rounding::same(13.0),
                    Stroke::new((1.0 - blink) * 3.0, i.color),
                );
                ui.painter()
                    .rect_stroke(rect, Rounding::same(13.0), Stroke::new(1.0, i.color));
            }
        }
    }
}
