use egui_extras::{TableBuilder, TableRow};
use ron::{
    extensions::Extensions,
    ser::{to_string_pretty, PrettyConfig},
};

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

const HEROES_FOLDER: &str = "/Users/admin/Documents/GitHub/arena-of-ideas/assets/ron/heroes";

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
                    if ui.button("Paste").clicked() {
                        if let Some(s) = get_from_clipboard(world) {
                            match ron::from_str(&s) {
                                Ok(u) => td.units.insert(0, u),
                                Err(e) => AlertPlugin::add_error(
                                    Some("Paste Failed".to_owned()),
                                    e.to_string(),
                                    None,
                                ),
                            }
                        }
                    }
                });

                let columns = Column::iter().collect_vec();
                ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                TableBuilder::new(ui)
                    .auto_shrink(false)
                    .striped(true)
                    .column(egui_extras::Column::exact(100.0))
                    .columns(egui_extras::Column::auto(), columns.len())
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
                            ui.style_mut().visuals.widgets.inactive.bg_stroke =
                                Stroke::new(1.0, white());
                            ui.horizontal(|ui| {
                                if ui.button("s").clicked() {
                                    for unit in &td.units {
                                        Self::save_unit(unit);
                                    }
                                }
                                if ui.button_red("-").clicked() {
                                    td.units.clear();
                                }
                            });
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
                        let houses: HashMap<String, Color32> = HashMap::from_iter(
                            pools
                                .houses
                                .iter()
                                .map(|(k, v)| (k.clone(), v.color.clone().into())),
                        );
                        let mut delete: Option<usize> = None;
                        for unit in td.units.iter_mut() {
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
                                for column in Column::iter() {
                                    column.show_row(unit, &houses, &mut row);
                                }
                                let i = row.index();
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button("s").clicked() {
                                            Self::save_unit(unit);
                                        }
                                        if ui.button_red("-").clicked() {
                                            delete = Some(i);
                                        }
                                        if ui.button("e").clicked() {
                                            HeroEditorPlugin::load_unit(unit.clone(), world);
                                            GameState::HeroEditor.change(world);
                                            return;
                                        }
                                    });
                                });
                            });
                        }
                        if let Some(delete) = delete {
                            td.units.remove(delete);
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
        if !pd.hero_table_data.eq(td) {
            mem::swap(&mut pd.hero_table_data, td);
            pd.save(world).unwrap();
        }
    }

    fn save_unit(unit: &PackedUnit) {
        if unit.name.is_empty() {
            AlertPlugin::add_error(None, "Can't save unit with empty name".to_owned(), None);
            return;
        }
        let path = format!("{HEROES_FOLDER}/{}.unit.ron", unit.name.to_lowercase());
        match std::fs::write(
            &path,
            to_string_pretty(
                &unit,
                PrettyConfig::new()
                    .extensions(Extensions::IMPLICIT_SOME)
                    .compact_arrays(true),
            )
            .unwrap(),
        ) {
            Ok(_) => {
                info!("Unit {} saved to {}", unit.name, &path);
            }
            Err(e) => {
                error!("Failed to save unit {}: {}", unit.name, e);
            }
        };
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
    House,
    Atk,
    Hp,
    Trigger,
    Target,
    Effect,
}

#[derive(Serialize, Deserialize, Clone, Debug, EnumIter, PartialEq)]
enum Sorting {
    Asc,
    Desc,
}

impl Column {
    fn show_row(
        self,
        unit: &mut PackedUnit,
        houses: &HashMap<String, Color32>,
        row: &mut TableRow,
    ) {
        let i = row.index();
        row.col(|ui| {
            match self {
                Column::Name => {
                    TextEdit::singleline(&mut unit.name)
                        .text_color(houses.get(&unit.houses).cloned().unwrap_or_default())
                        .ui(ui);
                }
                Column::House => {
                    let house: &mut String = &mut unit.houses;
                    ComboBox::from_id_source(Id::new(&unit.name).with(i))
                        .selected_text(
                            house
                                .clone()
                                .add_color(*houses.get(house).unwrap())
                                .rich_text(ui),
                        )
                        .width(140.0)
                        .show_ui(ui, |ui| {
                            for (h, c) in houses.iter().sorted_by_key(|(k, _)| k.to_owned()) {
                                let text = h.clone().add_color(*c).rich_text(ui);
                                ui.selectable_value(house, h.clone(), text);
                            }
                        });
                }
                Column::Atk => {
                    DragValue::new(&mut unit.atk).clamp_range(0..=99).ui(ui);
                }
                Column::Hp => {
                    DragValue::new(&mut unit.hp).clamp_range(0..=99).ui(ui);
                }
                Column::Trigger | Column::Target | Column::Effect => match &mut unit.trigger {
                    Trigger::Fire {
                        triggers,
                        targets,
                        effects,
                    } => {
                        let list = match &self {
                            Column::Trigger => triggers
                                .iter_mut()
                                .map(|(t, s)| (t.to_string(), s))
                                .collect_vec(),
                            Column::Target => targets
                                .iter_mut()
                                .map(|(t, s)| (t.to_string(), s))
                                .collect_vec(),
                            Column::Effect => effects
                                .iter_mut()
                                .map(|(t, s)| (t.to_string(), s))
                                .collect_vec(),
                            _ => panic!(),
                        };

                        ui.vertical(|ui| {
                            for (name, text) in list {
                                ui.horizontal(|ui| {
                                    let mut is_override = text.is_some();
                                    if ui.checkbox(&mut is_override, "").changed() {
                                        if is_override {
                                            *text = Some(default());
                                        } else {
                                            *text = None;
                                        }
                                    }
                                    if let Some(text) = text {
                                        TextEdit::singleline(text).ui(ui);
                                    } else {
                                        ui.label(name.to_string());
                                    }
                                });
                            }
                        });
                    }
                    _ => {}
                },
            };
        });
    }
}
