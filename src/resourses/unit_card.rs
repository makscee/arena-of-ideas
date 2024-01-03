use std::collections::VecDeque;

use bevy_egui::egui::Order;

use super::*;

#[derive(Clone, Debug, Component)]
pub struct UnitCard {
    pub name: ColoredString,
    pub stats: ColoredString,
    pub hp: i32,
    pub atk: i32,
    pub description: ColoredString,
    pub definitions: Vec<(ColoredString, ColoredString)>,
    pub statuses: Vec<(ColoredString, i32, ColoredString)>,
    pub open: bool,
    pub entity: Entity,
}

impl UnitCard {
    pub fn from_packed(unit: PackedUnit, world: &mut World) -> Result<Self> {
        let entity = unit.unpack(Faction::Left.team_entity(world), None, world);
        let card = Self::from_entity(entity, world)?;
        world.entity_mut(entity).despawn_recursive();
        Ok(card)
    }
    pub fn from_entity(entity: Entity, world: &World) -> Result<Self> {
        let t = get_play_head(world);
        let statuses = Status::collect_entity_statuses(entity, world)
            .into_iter()
            .filter_map(|e| {
                let state = &VarState::snapshot(e, world, t);
                let charges = state.get_int(VarName::Charges);
                if charges.is_err() || charges.is_ok_and(|c| c <= 0) {
                    return None;
                }
                let name = state.get_string_at(VarName::Name, t);
                if let Ok(name) = name {
                    let color: Color32 = Pools::get_status_house(&name, world)
                        .unwrap()
                        .color
                        .clone()
                        .into();
                    let description =
                        if let Some(status) = Pools::get_status(&name.to_string(), world) {
                            status.description.clone().to_colored().inject_vars(state)
                        } else {
                            ColoredString::default()
                        };
                    Some((
                        state
                            .get_string_at(VarName::Name, t)
                            .unwrap()
                            .add_color(color),
                        state.get_int_at(VarName::Charges, t).unwrap(),
                        description,
                    ))
                } else {
                    None
                }
            })
            .collect_vec();
        let state = VarState::try_get(entity, world)?;
        let description = state.get_string_at(VarName::Description, t)?;
        let mut definitions: Vec<(ColoredString, ColoredString)> = default();
        let mut added_definitions: HashSet<String> = default();
        let mut raw_definitions = VecDeque::from_iter(description.extract_bracketed(("[", "]")));
        while let Some(name) = raw_definitions.pop_front() {
            let (color, description) = if let Some(ability) = Pools::get_ability(&name, world) {
                let color: Color32 = Pools::get_ability_house(&name, world)
                    .unwrap()
                    .color
                    .clone()
                    .into();
                (color, ability.description.clone())
            } else if let Some(status) = Pools::get_status(&name, world) {
                let color: Color32 = Pools::get_status_house(&name, world)
                    .unwrap()
                    .color
                    .clone()
                    .into();
                (color, status.description.clone())
            } else {
                continue;
            };
            if !added_definitions.insert(name.clone()) {
                continue;
            }
            definitions.push((
                name.add_color(color),
                description
                    .to_colored()
                    .inject_definitions(world)
                    .inject_vars(&default()),
            ));
            raw_definitions.extend(description.extract_bracketed(("[", "]")));
        }

        let description = description
            .to_colored()
            .inject_vars(state)
            .inject_definitions(world);
        let hp = VarState::find_value(entity, VarName::Hp, t, world)?.get_int()?;
        let atk = VarState::find_value(entity, VarName::Atk, t, world)?.get_int()?;
        let name = state
            .get_string_at(VarName::Name, t)?
            .add_color(state.get_color_at(VarName::HouseColor, t)?.c32());
        let stats = format!(" {atk}/{hp}").add_color(white());

        Ok(Self {
            name,
            stats,
            hp,
            atk,
            description,
            statuses,
            entity,
            definitions,
            open: true,
        })
    }

    pub fn refresh(&mut self, world: &World) {
        *self = Self::from_entity(self.entity, world).unwrap();
    }

    fn show_status_lines(&self, show_desc: bool, ui: &mut Ui) {
        for (name, charges, description) in self.statuses.iter() {
            text_dots_text(name, &charges.to_string().add_color(white()), ui);
            if show_desc && !description.is_empty() {
                ui.vertical(|ui| {
                    ui.label(description.widget());
                });
            }
        }
    }

    pub fn show_name(&self, open: bool, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.style_mut().wrap = Some(true);
            if open {
                Label::new(self.name.widget_with_font(Some(
                    TextStyle::Name("Heading2".into()).resolve(ui.style()),
                )))
                .ui(ui);
                ui.label(self.stats.widget());
            } else {
                ui.label(self.name.widget());
            }
        });
    }

    pub fn show_frames(&self, open: bool, expanded: bool, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.show_name(open, ui);
            if !open {
                return;
            }
            frame(ui, |ui| {
                if !self.description.is_empty() {
                    ui.vertical(|ui| {
                        ui.label(self.description.widget());
                    });
                }
                if !expanded {
                    self.show_status_lines(false, ui);
                }
            });
            if !expanded {
                return;
            }
            for (name, text) in &self.definitions {
                frame(ui, |ui| {
                    ui.label(name.rich_text().family(FontFamily::Name("bold".into())));
                    ui.horizontal_wrapped(|ui| {
                        ui.label(text.widget());
                    });
                });
            }
            if !self.statuses.is_empty() {
                frame(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(RichText::new("Statuses").color(white()));
                    });
                    self.show_status_lines(open, ui);
                });
            }
        });
    }

    pub fn show_window(&self, open: bool, ctx: &egui::Context, world: &World) {
        if let Some(visibility) = world.get::<ComputedVisibility>(self.entity) {
            if !visibility.is_visible() {
                return;
            }
        }
        window("UNIT")
            .id(self.entity)
            .set_width(if open { 200.0 } else { 120.0 })
            .title_bar(false)
            .order(if open {
                egui::Order::Foreground
            } else {
                Order::Middle
            })
            .entity_anchor(self.entity, Align2::CENTER_TOP, vec2(0.0, 1.2), world)
            .show(ctx, |ui| {
                if open && world.get_entity(self.entity).is_some() {
                    Self::from_entity(self.entity, world)
                        .unwrap()
                        .show_frames(open, true, ui)
                } else {
                    self.show_frames(open, true, ui)
                }
            });
    }

    pub fn show_ui(&self, open: bool, expanded: bool, ui: &mut Ui) {
        window("UNIT")
            .id(self.entity)
            .set_width(if open { 200.0 } else { 120.0 })
            .title_bar(false)
            .show_ui(ui, |ui| self.show_frames(open, expanded, ui));
    }

    pub fn set_open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
}
