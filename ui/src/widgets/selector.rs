use std::cmp::Ordering;

use egui::{ComboBox, Key};

use super::*;

pub struct Selector {
    name: WidgetText,
}

impl Selector {
    pub fn new(name: impl Into<WidgetText>) -> Self {
        Self { name: name.into() }
    }
    pub fn ui_enum<E: ToCstr + AsRef<str> + IntoEnumIterator + Clone + PartialEq>(
        self,
        value: &mut E,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(self.name);
            let lookup_id = ui.id();
            if ComboBox::from_id_source(ui.next_auto_id())
                .selected_text(value.cstr().widget(1.0, ui))
                .show_ui(ui, |ui| {
                    let mut lookup = ui
                        .ctx()
                        .data(|r| r.get_temp::<String>(lookup_id))
                        .unwrap_or_default();
                    let mut take_first = false;
                    for e in ui.ctx().input(|i| i.events.clone()) {
                        match e {
                            egui::Event::Text(s) => lookup += &s.to_lowercase(),
                            egui::Event::Key {
                                key: Key::Backspace,
                                pressed: true,
                                ..
                            } => {
                                lookup.pop();
                            }
                            egui::Event::Key {
                                key: Key::Tab,
                                pressed: true,
                                ..
                            } => take_first = true,
                            _ => {}
                        }
                    }

                    ui.ctx().data_mut(|w| {
                        *w.get_temp_mut_or_default::<String>(lookup_id) = lookup.clone();
                    });
                    lookup.label(ui);
                    let variants = E::iter()
                        .sorted_by(|a, _| {
                            if a.as_ref().to_lowercase().contains(&lookup) {
                                Ordering::Less
                            } else {
                                Ordering::Greater
                            }
                        })
                        .collect_vec();
                    for e in variants {
                        let grayed_out =
                            !lookup.is_empty() && !e.as_ref().to_lowercase().contains(&lookup);
                        let text = if grayed_out {
                            e.as_ref().cstr_c(VISIBLE_DARK)
                        } else {
                            e.cstr()
                        }
                        .widget(1.0, ui);
                        let resp = ui.selectable_value(value, e.clone(), text);
                        if take_first && !grayed_out {
                            *value = e;
                            changed = true;
                            ui.memory_mut(|w| w.close_popup());
                            take_first = false;
                        }
                        changed |= resp.changed();
                    }
                })
                .response
                .clicked()
            {
                ui.ctx()
                    .data_mut(|w| w.insert_temp(dbg!(lookup_id), String::new()));
            };
        });
        changed
    }
    pub fn ui_iter<'a, E: PartialEq + Clone + ToString + ToCstr + 'a, I>(
        self,
        value: &mut E,
        values: I,
        ui: &mut Ui,
    ) -> bool
    where
        I: IntoIterator<Item = &'a E>,
    {
        let mut changed = false;
        ui.label(self.name.clone());
        ComboBox::from_id_source(self.name.text())
            .selected_text(value.cstr_c(name_color(&value.to_string())).widget(1.0, ui))
            .show_ui(ui, |ui| {
                for e in values {
                    let text = e.cstr_c(name_color(&e.to_string())).widget(1.0, ui);
                    changed |= ui.selectable_value(value, e.clone(), text).changed();
                }
            });
        changed
    }
}
