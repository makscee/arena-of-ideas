use serde::de::DeserializeOwned;

use super::*;

fn lookup_id() -> Id {
    static LOOKUP_STRING: OnceCell<Id> = OnceCell::new();
    *LOOKUP_STRING.get_or_init(|| Id::new("lookup_string"))
}
fn lookup_text(ctx: &egui::Context) -> String {
    ctx.data(|r| r.get_temp::<String>(lookup_id()).unwrap_or_default())
}
fn lookup_text_clear(ctx: &egui::Context) {
    ctx.data_mut(|w| w.remove_temp::<String>(lookup_id()));
}
fn lookup_text_push(ctx: &egui::Context, s: &str) {
    ctx.data_mut(|w| w.get_temp_mut_or_default::<String>(lookup_id()).push_str(s));
}
fn lookup_text_pop(ctx: &egui::Context) {
    ctx.data_mut(|w| w.get_temp_mut_or_default::<String>(lookup_id()).pop());
}
pub trait ShowEditor: ToCstr + Default + Serialize + DeserializeOwned + Clone {
    fn transparent() -> bool {
        true
    }
    fn get_variants() -> impl Iterator<Item = Self>;
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>>;
    fn wrapper() -> Option<Self> {
        None
    }
    fn show_content(&mut self, _context: &Context, _world: &mut World, _ui: &mut Ui) {}
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui);
    fn show_node(&mut self, name: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        const SHADOW: Shadow = Shadow {
            offset: egui::Vec2::ZERO,
            blur: 5.0,
            spread: 5.0,
            color: Color32::from_rgba_premultiplied(20, 20, 20, 25),
        };
        const FRAME_REGULAR: Frame = Frame {
            inner_margin: Margin::same(4.0),
            rounding: Rounding::same(6.0),
            shadow: SHADOW,
            outer_margin: Margin::ZERO,
            fill: BG_DARK,
            stroke: Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            },
        };
        const FRAME_TRANSPARENT: Frame = Frame {
            inner_margin: Margin::same(4.0),
            rounding: Rounding::same(6.0),
            shadow: Shadow::NONE,
            outer_margin: Margin::ZERO,
            fill: TRANSPARENT,
            stroke: Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            },
        };
        if !name.is_empty() {
            name.cstr_cs(VISIBLE_DARK, CstrStyle::Small).label(ui);
        }
        let resp = if Self::transparent() {
            FRAME_TRANSPARENT
        } else {
            FRAME_REGULAR
        }
        .show(ui, |ui| {
            ui.push_id(name, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.vertical(|ui| {
                        self.show_self(ui, world);
                        // ui.set_max_width(ui.min_size().x);
                        self.show_content(context, world, ui);
                    });
                    ui.vertical(|ui| {
                        self.show_children(context, world, ui);
                    });
                });
            });
        });
        if let Some(mut wrapper) = Self::wrapper() {
            let rect = resp.response.rect;
            let rect =
                Rect::from_center_size(rect.left_center(), egui::vec2(2.0, rect.height() - 13.0));
            let e_rect = rect.expand2(egui::vec2(4.0, 0.0));
            let ui = &mut ui.child_ui(rect, *ui.layout(), None);
            let resp = ui.allocate_rect(e_rect, Sense::click());
            let color = if resp.hovered() {
                YELLOW
            } else {
                VISIBLE_LIGHT
            };
            ui.painter().rect_filled(rect, Rounding::ZERO, color);
            if resp.clicked() {
                let mut inner = wrapper.get_inner_mut();
                if inner.is_empty() {
                    return;
                }
                *inner[0] = Box::new(self.clone());
                *self = wrapper;
            }
        }
    }
    fn show_self(&mut self, ui: &mut Ui, world: &mut World) {
        let widget = self.cstr().widget(1.0, ui);
        let variants = Self::get_variants();
        let resp = if variants.try_len().is_ok_and(|l| l > 0) {
            let resp = ui
                .menu_button(widget, |ui| {
                    lookup_text(ui.ctx()).cstr().label(ui);
                    let mut take_first = false;
                    for e in ui.ctx().input(|i| i.events.clone()) {
                        match e {
                            egui::Event::Text(s) => lookup_text_push(ui.ctx(), &s),
                            egui::Event::Key {
                                key: Key::Backspace,
                                pressed: true,
                                ..
                            } => lookup_text_pop(ui.ctx()),
                            egui::Event::Key {
                                key: Key::Tab,
                                pressed: true,
                                ..
                            } => take_first = true,
                            _ => {}
                        }
                    }
                    ui.set_min_height(300.0);
                    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        ui.reset_style();
                        let lookup = lookup_text(ui.ctx()).to_lowercase();
                        for e in Self::get_variants() {
                            let c = e.cstr();
                            if !lookup.is_empty()
                                && !c.get_text().to_lowercase().starts_with(&lookup)
                            {
                                continue;
                            }
                            if c.button(ui).clicked() || take_first {
                                self.replace_self(e);
                                ui.close_menu();
                                break;
                            }
                        }
                    });
                })
                .response;
            if resp.clicked() {
                lookup_text_clear(ui.ctx());
            }
            resp
        } else {
            ui.label(widget)
        };
        resp.context_menu(|ui| {
            ui.reset_style();
            if Button::new("Copy").ui(ui).clicked() {
                match ron::to_string(self) {
                    Ok(v) => {
                        copy_to_clipboard(&v, world);
                    }
                    Err(e) => format!("Failed to copy: {e}").notify_error(world),
                }
                ui.close_menu();
            }
            if Button::new("Paste").ui(ui).clicked() {
                if let Some(v) = paste_from_clipboard(world) {
                    match ron::from_str::<Self>(&v) {
                        Ok(v) => *self = v,
                        Err(e) => format!("Failed to paste text {v}: {e}").notify_error(world),
                    }
                } else {
                    format!("Clipboard is empty").notify_error(world);
                }
                ui.close_menu();
            }
        });
    }
    fn replace_self(&mut self, value: Self) {
        let mut inner = self
            .get_inner_mut()
            .into_iter()
            .map(|i| mem::take(i))
            .rev()
            .collect_vec();
        *self = value;
        for s in self.get_inner_mut() {
            if let Some(i) = inner.pop() {
                *s = i;
            } else {
                break;
            }
        }
    }
}
