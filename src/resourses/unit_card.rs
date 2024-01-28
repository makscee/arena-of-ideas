use std::collections::VecDeque;

use bevy_egui::egui::Order;

use super::*;

impl VarState {
    fn name(&self, open: bool) -> Result<ColoredString> {
        let t = get_play_head();
        let level = self.get_int_at(VarName::Level, t)?;
        let mut result = ColoredString::default();
        for (i, name) in self.get_string_at(VarName::Name, t)?.split("+").enumerate() {
            let var = match i {
                0 => VarName::HouseColor1,
                1 => VarName::HouseColor2,
                2 => VarName::HouseColor3,
                _ => panic!("Too many houses"),
            };
            result
                .push(name.to_owned(), self.get_color_at(var, t)?.c32())
                .set_style(match open {
                    true => ColoredStringStyle::Heading2,
                    false => ColoredStringStyle::Normal,
                });
        }
        result
            .push_colored(
                " lv."
                    .to_colored()
                    .set_style(ColoredStringStyle::Small)
                    .take(),
            )
            .push_colored(
                level
                    .to_string()
                    .add_color(match level {
                        1 => white(),
                        2 => yellow(),
                        _ => red(),
                    })
                    .set_style(ColoredStringStyle::Bold)
                    .take(),
            );
        Ok(result)
    }
    fn description(&self, world: &World) -> Result<ColoredString> {
        let t = get_play_head();
        let description = self
            .get_string_at(VarName::Description, t)?
            .to_colored()
            .inject_trigger(self)
            .inject_vars(self)
            .inject_definitions(world);
        Ok(description)
    }
    fn extra_lines(&self) -> Result<Vec<(ColoredString, ColoredString)>> {
        let t = get_play_head();
        let level = self.get_int_at(VarName::Level, t)?;
        let stacks = self.get_int_at(VarName::Stacks, t)?;
        let stacks = format!("{stacks}/{} ", level + 1);
        let level_line = (
            "level".to_colored(),
            stacks
                .to_colored()
                .push_styled(level.to_string(), white(), ColoredStringStyle::Bold)
                .take(),
        );
        let atk = self.get_int_at(VarName::Atk, t)?;
        let hp = self.get_int_at(VarName::Hp, t)?;
        let hp_line = (
            "hp".to_colored(),
            hp.to_string()
                .add_color(red())
                .set_style(ColoredStringStyle::Bold)
                .take(),
        );
        let atk_line = (
            "atk".to_colored(),
            atk.to_string()
                .add_color(yellow())
                .set_style(ColoredStringStyle::Bold)
                .take(),
        );
        Ok([level_line, atk_line, hp_line].into())
    }
    fn statuses(
        &self,
        statuses: Vec<(String, i32)>,
        world: &World,
    ) -> Result<Vec<(ColoredString, i32, ColoredString)>> {
        let lines = statuses
            .into_iter()
            .map(|(name, charges)| {
                let color: Color32 = Pools::get_status_house(&name, world)
                    .unwrap()
                    .color
                    .clone()
                    .into();
                let state = VarState::new_with(VarName::Charges, VarValue::Int(charges));
                let description = if let Some(status) = Pools::get_status(&name.to_string(), world)
                {
                    status.description.clone().to_colored().inject_vars(&state)
                } else {
                    ColoredString::default()
                };
                (name.add_color(color), charges, description)
            })
            .collect_vec();
        if lines.is_empty() {
            return Err(anyhow!("No statuses"));
        }
        Ok(lines)
    }
    fn definitions(&self, world: &World) -> Result<Vec<(ColoredString, ColoredString)>> {
        let t = get_play_head();
        let description = self.get_string_at(VarName::EffectDescription, t)?;
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
                name.add_color(color)
                    .set_style(ColoredStringStyle::Bold)
                    .take(),
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
                    description.label(ui);
                });
            }
        }
    }

    fn show_name(name: ColoredString, ui: &mut Ui) {
        ui.vertical_centered(|ui| name.label(ui));
    }

    fn show_extra_lines(lines: Vec<(ColoredString, ColoredString)>, ui: &mut Ui) {
        for (name, value) in lines.iter() {
            text_dots_text(name, value, ui);
        }
    }

    fn show_frames(
        &self,
        statuses: Vec<(String, i32)>,
        open: bool,
        ui: &mut Ui,
        world: &World,
    ) -> Result<()> {
        let name = self.name(open)?;
        Self::show_name(name, ui);
        if !open {
            return Ok(());
        }
        let expanded = ui.input(|i| i.modifiers.shift) || SettingsData::get(world).expanded_hint;
        let description = self.description(world);
        let statuses = self.statuses(statuses, world);
        let extra_lines = self.extra_lines();
        frame(ui, |ui| {
            if let Ok(description) = description {
                ui.vertical(|ui| {
                    description.label(ui);
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
            ui.vertical_centered(|ui| {
                "SHIFT to expand"
                    .to_colored()
                    .set_style(ColoredStringStyle::Small)
                    .label(ui);
            });
            return Ok(());
        }
        let definition = self.definitions(world);
        if let Ok(definitions) = definition {
            for (name, text) in &definitions {
                frame(ui, |ui| {
                    name.label(ui);
                    ui.horizontal_wrapped(|ui| {
                        text.label(ui);
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

        Ok(())
    }

    pub fn show_state_card_window(
        &self,
        id: impl std::hash::Hash,
        statuses: Vec<(String, i32)>,
        open: bool,
        ctx: &egui::Context,
        world: &World,
    ) {
        window("UNIT")
            .id(id)
            .set_width(if open { 200.0 } else { 140.0 })
            .title_bar(false)
            .order(if open {
                egui::Order::Foreground
            } else {
                Order::Middle
            })
            .show(ctx, |ui| {
                let _ = self.show_frames(statuses, open, ui, world);
            });
    }

    pub fn show_state_card_ui(
        &self,
        id: impl std::hash::Hash,
        statuses: Vec<(String, i32)>,
        open: bool,
        ui: &mut Ui,
        world: &World,
    ) {
        window("UNIT")
            .id(id)
            .set_width(if open { 200.0 } else { 140.0 })
            .title_bar(false)
            .order(if open {
                egui::Order::Foreground
            } else {
                Order::Middle
            })
            .show_ui(ui, |ui| {
                let _ = self.show_frames(statuses, open, ui, world);
            });
    }

    pub fn show_entity_card_window(
        &self,
        entity: Entity,
        statuses: Vec<(String, i32)>,
        open: bool,
        ctx: &egui::Context,
        world: &World,
    ) {
        if let Some(visibility) = world.get::<ComputedVisibility>(entity) {
            if !visibility.is_visible() {
                return;
            }
        }
        window("UNIT")
            .id(entity)
            .set_width(if open { 200.0 } else { 140.0 })
            .title_bar(false)
            .order(if open {
                egui::Order::Foreground
            } else {
                Order::Middle
            })
            .entity_anchor(entity, Align2::CENTER_TOP, vec2(0.0, 1.2), world)
            .show(ctx, |ui| {
                let _ = self.show_frames(statuses, open, ui, world);
            });
    }

    pub fn show_entity_card_ui(
        &self,
        entity: Entity,
        statuses: Vec<(String, i32)>,
        open: bool,
        ui: &mut Ui,
        world: &World,
    ) {
        window("UNIT")
            .id(entity)
            .set_width(if open { 200.0 } else { 120.0 })
            .title_bar(false)
            .show_ui(ui, |ui| {
                let _ = self.show_frames(statuses, open, ui, world);
            });
    }
}
