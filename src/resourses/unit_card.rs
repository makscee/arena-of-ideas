use std::collections::VecDeque;

use bevy_egui::egui::Order;

use super::*;

impl VarState {
    fn name(&self) -> Result<ColoredString> {
        let t = get_play_head();
        let level = self.get_int_at(VarName::Level, t)?;
        let stacks = self.get_int_at(VarName::Stacks, t)?;
        Ok(self
            .get_string_at(VarName::Name, t)?
            .add_color(self.get_color_at(VarName::HouseColor, t)?.c32())
            .push(format!(" {}", level), white())
            .push(format!(" {stacks}/{}", level + 1), light_gray())
            .take())
    }
    fn description(&self, world: &World) -> Result<ColoredString> {
        let t = get_play_head();
        let description = self.get_string_at(VarName::Description, t)?;
        if description.is_empty() {
            return Err(anyhow!("No description"));
        }
        Ok(description
            .to_colored()
            .inject_vars(self)
            .inject_definitions(world))
    }
    fn extra_lines(&self) -> Result<Vec<(ColoredString, ColoredString)>> {
        let t = get_play_head();
        let level = self.get_int_at(VarName::Level, t)?;
        let level_line = (
            "level".add_color(light_gray()),
            level.to_string().add_color(white()),
        );
        let stacks = self.get_int_at(VarName::Stacks, t)?;
        let stacks_line = (
            "stacks".add_color(light_gray()),
            format!("{stacks}/{}", level + 1).add_color(white()),
        );
        Ok([level_line, stacks_line].into())
    }
    fn statuses(
        &self,
        entity: Entity,
        world: &World,
    ) -> Result<Vec<(ColoredString, i32, ColoredString)>> {
        let t = get_play_head();
        let statuses = Status::collect_entity_statuses(entity, world);
        let lines = statuses
            .into_iter()
            .filter_map(|e| {
                let state = &VarState::snapshot(e, world, t);
                let charges = state.get_int(VarName::Charges);
                if charges.is_err() || charges.is_ok_and(|c| c <= 0) {
                    return None;
                }
                let name = self.get_string_at(VarName::Name, t);
                if let Ok(name) = name {
                    let color: Color32 = Pools::get_status_house(&name, world)
                        .unwrap()
                        .color
                        .clone()
                        .into();
                    let description =
                        if let Some(status) = Pools::get_status(&name.to_string(), world) {
                            status.description.clone().to_colored().inject_vars(self)
                        } else {
                            ColoredString::default()
                        };
                    Some((
                        self.get_string_at(VarName::Name, t)
                            .unwrap()
                            .add_color(color),
                        self.get_int_at(VarName::Charges, t).unwrap(),
                        description,
                    ))
                } else {
                    None
                }
            })
            .collect_vec();
        if lines.is_empty() {
            return Err(anyhow!("No statuses"));
        }
        Ok(lines)
    }
    fn definitions(&self, world: &World) -> Result<Vec<(ColoredString, ColoredString)>> {
        let t = get_play_head();
        let description = self.get_string_at(VarName::Description, t)?;
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
        Ok(definitions)
    }

    fn show_status_lines(
        statuses: &[(ColoredString, i32, ColoredString)],
        show_desc: bool,
        ui: &mut Ui,
    ) {
        for (name, charges, description) in statuses.iter() {
            text_dots_text(name, &charges.to_string().add_color(white()), ui);
            if show_desc && !description.is_empty() {
                ui.vertical(|ui| {
                    ui.label(description.widget());
                });
            }
        }
    }

    fn show_name(name: ColoredString, open: bool, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.style_mut().wrap = Some(true);
            if open {
                Label::new(name.widget_with_font(Some(
                    TextStyle::Name("Heading2".into()).resolve(ui.style()),
                )))
                .ui(ui);
                // ui.label(self.stats.widget());
            } else {
                ui.label(name.widget());
            }
        });
    }

    fn show_extra_lines(lines: Vec<(ColoredString, ColoredString)>, ui: &mut Ui) {
        for (name, value) in lines.iter() {
            text_dots_text(name, value, ui);
        }
    }

    fn show_frames(
        &self,
        entity: Entity,
        open: bool,
        expanded: bool,
        ui: &mut Ui,
        world: &World,
    ) -> Result<()> {
        let description = self.description(world);
        let definition = self.definitions(world);
        let statuses = self.statuses(entity, world);
        let name = self.name()?;
        let extra_lines = self.extra_lines();
        ui.vertical(|ui| {
            Self::show_name(name, open, ui);
            if !open {
                return;
            }
            frame(ui, |ui| {
                if let Ok(description) = description {
                    ui.vertical(|ui| {
                        ui.label(description.widget());
                    });
                }
                if !expanded {
                    if let Ok(statuses) = statuses.as_ref() {
                        Self::show_status_lines(statuses, false, ui);
                    }
                }
            });
            if let Ok(lines) = extra_lines {
                frame(ui, |ui| {
                    Self::show_extra_lines(lines, ui);
                });
            }
            if !expanded {
                return;
            }
            if let Ok(definitions) = definition {
                for (name, text) in &definitions {
                    frame(ui, |ui| {
                        ui.label(name.rich_text().family(FontFamily::Name("bold".into())));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(text.widget());
                        });
                    });
                }
            }
            if let Ok(statuses) = statuses.as_ref() {
                frame(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(RichText::new("Statuses").color(white()));
                    });
                    Self::show_status_lines(statuses, open, ui);
                });
            }
        });
        Ok(())
    }

    pub fn show_window(&self, entity: Entity, open: bool, ctx: &egui::Context, world: &World) {
        if let Some(visibility) = world.get::<ComputedVisibility>(entity) {
            if !visibility.is_visible() {
                return;
            }
        }
        window("UNIT")
            .id(entity)
            .set_width(if open { 200.0 } else { 120.0 })
            .title_bar(false)
            .order(if open {
                egui::Order::Foreground
            } else {
                Order::Middle
            })
            .entity_anchor(entity, Align2::CENTER_TOP, vec2(0.0, 1.2), world)
            .show(ctx, |ui| {
                let _ = self.show_frames(entity, open, true, ui, world);
            });
    }

    pub fn show_ui(&self, entity: Entity, open: bool, expanded: bool, ui: &mut Ui, world: &World) {
        window("UNIT")
            .id(entity)
            .set_width(if open { 200.0 } else { 120.0 })
            .title_bar(false)
            .show_ui(ui, |ui| {
                let _ = self.show_frames(entity, open, expanded, ui, world);
            });
    }
}
