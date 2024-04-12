use egui_extras::TableBuilder;

use super::*;

pub struct HeroTablePlugin;

impl Plugin for HeroTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::ui
                .run_if(in_state(GameState::HeroTable))
                .after(PanelsPlugin::ui),
        );
    }
}

impl HeroTablePlugin {
    fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };

        let mut pd = PersistentData::load(world);
        let td = &mut pd.hero_table_data.clone();
        TopBottomPanel::new(egui::panel::TopBottomSide::Top, "table")
            .exact_height(ctx.available_rect().height())
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button("Load local").clicked() {
                        Pools::get_mut(world).heroes.clear();
                        PoolsPlugin::setup_heroes(world);
                        td.units = Pools::get(world).heroes.values().cloned().collect_vec();
                    }
                });

                let columns = Column::iter().collect_vec();
                let style = ui.style_mut();
                style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                TableBuilder::new(ui)
                    .striped(true)
                    .auto_shrink(false)
                    .columns(egui_extras::Column::auto(), columns.len() + 1)
                    .header(20.0, |mut h| {
                        let mut do_sort = false;
                        for column in columns.iter() {
                            h.col(|ui| {
                                if ui.button(column.to_string()).clicked() {
                                    do_sort = true;
                                    td.sorting = Some((
                                        *column,
                                        if td
                                            .sorting
                                            .as_ref()
                                            .is_some_and(|(_, s)| matches!(s, Sorting::Asc))
                                        {
                                            Sorting::Desc
                                        } else {
                                            Sorting::Asc
                                        },
                                    ));
                                }
                            });
                        }
                        h.col(|ui| {
                            ui.label("action");
                        });
                        if do_sort {
                            match td.sorting.as_ref().unwrap().0 {
                                Column::Name => td.units.sort_by_cached_key(|u| u.name.to_owned()),
                                Column::Hp => td.units.sort_by_cached_key(|u| u.hp),
                                Column::Atk => td.units.sort_by_cached_key(|u| u.atk),
                                Column::House => {
                                    td.units.sort_by_cached_key(|u| u.houses.to_owned())
                                }
                                Column::Trigger => {
                                    td.units.sort_by_cached_key(|u| match &u.trigger {
                                        Trigger::Fire {
                                            triggers,
                                            targets: _,
                                            effects: _,
                                        } => {
                                            if let Some((trigger, _)) = triggers.get(0) {
                                                trigger.to_string()
                                            } else {
                                                default()
                                            }
                                        }
                                        _ => default(),
                                    })
                                }
                                Column::Target => {
                                    td.units.sort_by_cached_key(|u| match &u.trigger {
                                        Trigger::Fire {
                                            triggers: _,
                                            targets,
                                            effects: _,
                                        } => {
                                            if let Some((target, _)) = targets.get(0) {
                                                target.to_string()
                                            } else {
                                                default()
                                            }
                                        }
                                        _ => default(),
                                    })
                                }
                                Column::Effect => {
                                    td.units.sort_by_cached_key(|u| match &u.trigger {
                                        Trigger::Fire {
                                            triggers: _,
                                            targets: _,
                                            effects,
                                        } => {
                                            if let Some((effect, _)) = effects.get(0) {
                                                effect.to_string()
                                            } else {
                                                default()
                                            }
                                        }
                                        _ => default(),
                                    })
                                }
                            }
                            if td.sorting.as_ref().unwrap().1 == Sorting::Desc {
                                td.units.reverse();
                            }
                        }
                    })
                    .body(|mut body| {
                        let pools = Pools::get(world);
                        let houses: HashMap<String, Color> = HashMap::from_iter(
                            pools
                                .houses
                                .iter()
                                .map(|(k, v)| (k.clone(), v.color.clone().into())),
                        );
                        for (i, unit) in td.units.iter_mut().enumerate() {
                            let height = 20.0
                                * match &unit.trigger {
                                    Trigger::Fire {
                                        triggers,
                                        targets,
                                        effects,
                                    } => triggers.len().max(targets.len()).max(effects.len()),
                                    Trigger::List(list) => list.len(),
                                    Trigger::Change { .. } => 1,
                                }
                                .max(1) as f32;
                            body.row(height, |mut row| {
                                row.col(|ui| {
                                    TextEdit::singleline(&mut unit.name)
                                        .text_color(
                                            pools
                                                .house_color(&unit.houses)
                                                .unwrap_or_default()
                                                .c32(),
                                        )
                                        .ui(ui);
                                });
                                row.col(|ui| {
                                    DragValue::new(&mut unit.hp).clamp_range(0..=99).ui(ui);
                                });
                                row.col(|ui| {
                                    DragValue::new(&mut unit.atk).clamp_range(0..=99).ui(ui);
                                });

                                row.col(|ui| {
                                    let house: &mut String = &mut unit.houses;
                                    ComboBox::from_id_source(Id::new(&unit.name).with(i))
                                        .selected_text(house.clone())
                                        .width(140.0)
                                        .show_ui(ui, |ui| {
                                            for (h, _) in
                                                houses.iter().sorted_by_key(|(k, _)| k.to_owned())
                                            {
                                                ui.selectable_value(house, h.clone(), h.clone());
                                            }
                                        });
                                });
                                match &mut unit.trigger {
                                    Trigger::Fire {
                                        triggers,
                                        targets,
                                        effects,
                                    } => {
                                        row.col(|ui| {
                                            ui.vertical(|ui| {
                                                for (trigger, text) in triggers {
                                                    ui.horizontal(|ui| {
                                                        let mut is_override = text.is_some();
                                                        if ui
                                                            .checkbox(&mut is_override, "")
                                                            .changed()
                                                        {
                                                            if is_override {
                                                                *text = Some(default());
                                                            } else {
                                                                *text = None;
                                                            }
                                                        }
                                                        if let Some(text) = text {
                                                            TextEdit::singleline(text).ui(ui);
                                                        } else {
                                                            ui.label(trigger.to_string());
                                                        }
                                                    });
                                                }
                                            });
                                        });

                                        row.col(|ui| {
                                            ui.vertical(|ui| {
                                                for (target, text) in targets {
                                                    ui.horizontal(|ui| {
                                                        let mut is_override = text.is_some();
                                                        if ui
                                                            .checkbox(&mut is_override, "")
                                                            .changed()
                                                        {
                                                            if is_override {
                                                                *text = Some(default());
                                                            } else {
                                                                *text = None;
                                                            }
                                                        }
                                                        if let Some(text) = text {
                                                            TextEdit::singleline(text).ui(ui);
                                                        } else {
                                                            ui.label(target.to_string());
                                                        }
                                                    });
                                                }
                                            });
                                        });
                                        row.col(|ui| {
                                            ui.vertical(|ui| {
                                                for (effect, text) in effects {
                                                    ui.horizontal(|ui| {
                                                        let mut is_override = text.is_some();
                                                        if ui
                                                            .checkbox(&mut is_override, "")
                                                            .changed()
                                                        {
                                                            if is_override {
                                                                *text = Some(default());
                                                            } else {
                                                                *text = None;
                                                            }
                                                        }
                                                        if let Some(text) = text {
                                                            TextEdit::singleline(text).ui(ui);
                                                        } else {
                                                            let text = effect.to_string();
                                                            ui.label(text.clone())
                                                                .on_hover_text(text);
                                                        }
                                                    });
                                                }
                                            });
                                        });
                                    }
                                    _ => {}
                                }
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("u").clicked() {}
                                        if ui.button("s").clicked() {
                                            let path = format!("/Users/admin/Documents/GitHub/arena-of-ideas/assets/ron/heroes/{}.unit.ron", unit.name.to_lowercase());
                                            match std::fs::write(&path, ron::to_string(&unit).unwrap()) {
                                                Ok(_) => {
                                                    info!("Unit {} saved to {}", unit.name, &path);
                                                }
                                                Err(e) => {
                                                    error!("Failed to save unit {}: {}", unit.name, e);
                                                }
                                            };
                                        }
                                        if ui.button_red("-").clicked() {}
                                    });
                                });
                            });
                        }
                        body.row(50.0, |mut row| {
                            row.col(|ui| {
                                if ui.button("+").clicked() {
                                    td.units.push(default());
                                }
                            });
                        });
                    });
            });

        TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bot btns").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Editor").clicked() {
                    GameState::HeroEditor.change(world);
                }
            });
        });
        if !pd.hero_table_data.eq(td) {
            mem::swap(&mut pd.hero_table_data, td);
            pd.save(world).unwrap();
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct HeroTableData {
    units: Vec<PackedUnit>,
    sorting: Option<(Column, Sorting)>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, EnumIter, PartialEq, Display)]
enum Column {
    Name,
    Hp,
    Atk,
    House,
    Trigger,
    Target,
    Effect,
}

#[derive(Serialize, Deserialize, Clone, Debug, EnumIter, PartialEq)]
enum Sorting {
    Asc,
    Desc,
}
