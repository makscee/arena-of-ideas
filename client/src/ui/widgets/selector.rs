use super::*;
use bevy_egui::egui::Popup;
use egui::ComboBox;

pub struct Selector;

impl Selector {
    fn fuzzy_match(text: &str, pattern: &str) -> Option<usize> {
        let text_lower = text.to_lowercase();

        if pattern.is_empty() {
            return Some(0);
        }

        let mut pattern_chars = pattern.chars().peekable();
        let mut score = 0;
        let mut last_match_pos = 0;

        for (pos, c) in text_lower.chars().enumerate() {
            if Some(c) == pattern_chars.peek().copied() {
                pattern_chars.next();
                let gap = if last_match_pos == 0 {
                    pos
                } else {
                    pos - last_match_pos
                };
                score += if gap == 1 { 10 } else { 5 };
                last_match_pos = pos;
            }
        }

        if pattern_chars.peek().is_none() {
            Some(score)
        } else {
            None
        }
    }

    pub fn ui_enum<E: ToCstr + AsRef<str> + IntoEnumIterator + Clone + PartialEq>(
        value: &mut E,
        ui: &mut Ui,
    ) -> (Option<E>, Response) {
        let mut changed = None;
        let lookup_id = ui.id();
        let combo_response = ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(value.cstr().widget(1.0, ui.style()))
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
                    .sorted_by(|a, b| {
                        let a_match = Self::fuzzy_match(a.as_ref(), &lookup);
                        let b_match = Self::fuzzy_match(b.as_ref(), &lookup);
                        match (a_match, b_match) {
                            (Some(a_score), Some(b_score)) => b_score.cmp(&a_score),
                            (Some(_), None) => Ordering::Less,
                            (None, Some(_)) => Ordering::Greater,
                            (None, None) => Ordering::Equal,
                        }
                    })
                    .collect_vec();
                for e in variants {
                    let is_match =
                        lookup.is_empty() || Self::fuzzy_match(e.as_ref(), &lookup).is_some();
                    let grayed_out = !lookup.is_empty() && !is_match;
                    let text = if grayed_out {
                        e.as_ref().cstr_c(ui.visuals().weak_text_color())
                    } else {
                        e.cstr()
                    }
                    .widget(1.0, ui.style());
                    let mut response = ui.selectable_label(*value == e, text);
                    if response.clicked() && *value != e {
                        changed = Some(value.clone());
                        *value = e;
                        response.mark_changed();
                    } else if take_first && !grayed_out {
                        changed = Some(value.clone());
                        *value = e;
                        Popup::close_all(ui.ctx());
                        take_first = false;
                    }
                }
            });
        let mut response = combo_response.response;
        if response.clicked() {
            ui.ctx()
                .data_mut(|w| w.insert_temp(lookup_id, String::new()));
        };
        if changed.is_some() {
            response.mark_changed();
        }
        (changed, response)
    }
    pub fn ui_iter<'a, E: PartialEq + Clone + ToCstr + 'a, I>(
        value: &mut E,
        values: I,
        ui: &mut Ui,
    ) -> (bool, Response)
    where
        I: IntoIterator<Item = &'a E>,
    {
        let mut changed = false;
        let combo_response = ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(
                value
                    .cstr_c(name_color(&value.cstr().to_string()))
                    .widget(1.0, ui.style()),
            )
            .show_ui(ui, |ui| {
                for e in values {
                    let text = e
                        .cstr_c(name_color(&e.cstr().to_string()))
                        .widget(1.0, ui.style());
                    if ui.selectable_value(value, e.clone(), text).changed() {
                        changed = true;
                    }
                }
            });
        (changed, combo_response.response)
    }
}
