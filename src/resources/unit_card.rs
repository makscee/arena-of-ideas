use std::collections::VecDeque;

use bevy_egui::egui::Order;

use super::*;

impl VarState {
    fn name(&self, open: bool) -> Result<ColoredString> {
        let t = GameTimer::get().play_head();
        let level = self.get_int_at(VarName::Level, t)?;
        let mut result = ColoredString::default();
        let names = self
            .get_string_at(VarName::Name, t)?
            .split('+')
            .map(|v| v.to_string())
            .collect_vec();
        let len = names.len();
        for (i, name) in names.into_iter().enumerate() {
            let name = match i {
                0 => {
                    if len > 1 {
                        name.split_at(name.len() / 2).0
                    } else {
                        name.as_str()
                    }
                }
                1 => {
                    if len > 2 {
                        name.split_at(name.len() / 2).0
                    } else {
                        name.split_at(name.len() / 2).1
                    }
                }
                2 => name.split_at(name.len() / 2).1,
                _ => name.as_str(),
            };
            let var = match i {
                0 => VarName::HouseColor1,
                1 => VarName::HouseColor2,
                2 => VarName::HouseColor3,
                _ => panic!("Too many houses"),
            };
            result
                .push(name.to_owned(), self.get_color_at(var, t)?.c32())
                .set_style_ref(match open {
                    true => ColoredStringStyle::Heading2,
                    false => ColoredStringStyle::Normal,
                });
        }
        result
            .push_colored(
                " lv."
                    .to_colored()
                    .set_style_ref(ColoredStringStyle::Small)
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
                    .set_style_ref(ColoredStringStyle::Bold)
                    .take(),
            );
        Ok(result)
    }
    fn rarity(&self) -> Result<ColoredString> {
        let r = Rarity::from_repr(self.get_int(VarName::Rarity)?).context("No rarity for disc")?;
        Ok(r.as_ref().add_color(r.color()))
    }
    fn description(&self, expanded: bool, world: &World) -> Result<Vec<(Icon, ColoredString)>> {
        let t = GameTimer::get().play_head();
        let mut result: Vec<(Icon, ColoredString)> = default();
        if let Ok(text) = self.get_string_at(VarName::TriggerDescription, t) {
            if !text.is_empty() {
                let mut text = text
                    .to_colored()
                    .set_style(ColoredStringStyle::Normal)
                    .inject_vars(self)
                    .inject_definitions(world);
                if expanded {
                    text.push_colored_front(
                        "trigger: "
                            .add_color(white())
                            .set_style(ColoredStringStyle::Normal),
                    );
                }
                result.push((Icon::Lightning, text));
            }
        }
        if let Ok(text) = self.get_string_at(VarName::TargetDescription, t) {
            if !text.is_empty() {
                let mut text = text
                    .to_colored()
                    .set_style(ColoredStringStyle::Normal)
                    .inject_vars(self)
                    .inject_definitions(world);
                if expanded {
                    text.push_colored_front(
                        "target: "
                            .add_color(white())
                            .set_style(ColoredStringStyle::Normal),
                    );
                }
                result.push((Icon::Target, text));
            }
        }
        if let Ok(text) = self.get_string_at(VarName::EffectDescription, t) {
            if !text.is_empty() {
                let mut text = text
                    .to_colored()
                    .set_style(ColoredStringStyle::Normal)
                    .inject_vars(self)
                    .inject_definitions(world);
                if expanded {
                    text.push_colored_front(
                        "effect: "
                            .add_color(white())
                            .set_style(ColoredStringStyle::Normal),
                    );
                }
                result.push((Icon::Flame, text));
            }
        }
        Ok(result)
    }
    fn houses(&self, world: &World) -> Result<ColoredString> {
        let t = GameTimer::get().play_head();
        let mut result = ColoredString::default();
        for house in self.get_string_at(VarName::Houses, t)?.split("+") {
            let color = Pools::get_house_color(house, world)
                .with_context(|| format!("House {house} not found"))?;
            result.push_colored(format!("{house} ").add_color(color.c32()));
        }
        Ok(result)
    }
    fn extra_lines(&self) -> Result<Vec<(ColoredString, ColoredString)>> {
        let t = GameTimer::get().play_head();
        let stacks = self.get_int_at(VarName::Stacks, t)?;
        let (level, to_next) = PackedUnit::level_from_stacks(stacks);
        let to_next = format!("({} stack to next) ", to_next);
        let level_line = (
            "level".to_colored(),
            to_next
                .to_colored()
                .push_styled(level.to_string(), white(), ColoredStringStyle::Bold)
                .take(),
        );
        let stacks = self.get_int_at(VarName::Stacks, t)?;
        let pwr = self.get_int_at(VarName::Pwr, t)?;
        let hp = self.get_int_at(VarName::Hp, t)?;
        let stacks_line = (
            "stacks".to_colored(),
            stacks
                .to_string()
                .add_color(white())
                .set_style_ref(ColoredStringStyle::Bold)
                .take(),
        );
        let hp_line = (
            "hp".to_colored(),
            hp.to_string()
                .add_color(red())
                .set_style_ref(ColoredStringStyle::Bold)
                .take(),
        );
        let atk_line = (
            "pwr".to_colored(),
            pwr.to_string()
                .add_color(yellow())
                .set_style_ref(ColoredStringStyle::Bold)
                .take(),
        );
        Ok([level_line, stacks_line, atk_line, hp_line].into())
    }
    fn statuses(
        &self,
        statuses: &Vec<(String, i32)>,
        world: &World,
    ) -> Result<Vec<(ColoredString, i32, ColoredString)>> {
        let lines = statuses
            .into_iter()
            .filter_map(
                |(name, charges)| match Pools::get_status_house(&name, world) {
                    Some(h) => {
                        let color = h.color.clone().into();
                        let state = VarState::new_with(VarName::Charges, VarValue::Int(*charges));
                        let description =
                            if let Some(status) = Pools::get_status(&name.to_string(), world) {
                                status
                                    .description
                                    .clone()
                                    .to_colored()
                                    .inject_vars(&state)
                                    .inject_definitions(world)
                            } else {
                                ColoredString::default()
                            };
                        Some((name.add_color(color), *charges, description))
                    }
                    None => None,
                },
            )
            .collect_vec();
        if lines.is_empty() {
            return Err(anyhow!("No statuses"));
        }
        Ok(lines)
    }
    fn definitions(
        &self,
        statuses: &Vec<(String, i32)>,
        world: &World,
    ) -> Result<Vec<(ColoredString, ColoredString)>> {
        let t = GameTimer::get().play_head();
        let description = self.get_string_at(VarName::EffectDescription, t)?
            + &self.get_string_at(VarName::TriggerDescription, t)?
            + &self.get_string_at(VarName::TargetDescription, t)?;
        let mut definitions: Vec<(ColoredString, ColoredString)> = default();
        let mut added_definitions: HashSet<String> = default();
        let mut raw_definitions = VecDeque::from_iter(
            description
                .extract_bracketed(("[", "]"))
                .into_iter()
                .chain(statuses.iter().map(|(s, _)| s.to_owned()))
                .unique(),
        );
        while let Some(name) = raw_definitions.pop_front() {
            let (color, description) = if let Some(ability) = Pools::get_ability(&name, world) {
                let color: Color32 = Pools::get_color_by_name(&name, world)?.c32();
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
            raw_definitions.extend(description.extract_bracketed(("[", "]")));
            let name_colored = name
                .add_color(color)
                .set_style_ref(ColoredStringStyle::Bold)
                .take();
            let description = description.to_colored();
            let vars = self
                .parent(world)
                .and_then(|s| s.get_faction(VarName::Faction).ok())
                .and_then(|f| TeamPlugin::get_ability_state(f, &name, world));
            definitions.push((
                name_colored,
                description
                    .inject_vars(vars.unwrap_or(&default()))
                    .inject_definitions(world),
            ));
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

    fn show_rarity(rarity: ColoredString, ui: &mut Ui) {
        ui.vertical_centered(|ui| rarity.label(ui));
    }

    fn show_houses(houses: ColoredString, ui: &mut Ui) {
        ui.vertical_centered(|ui| houses.label(ui));
    }

    fn show_extra_lines(lines: Vec<(ColoredString, ColoredString)>, ui: &mut Ui) {
        for (name, value) in lines.iter() {
            text_dots_text(name, value, ui);
        }
    }

    fn show_description(lines: Vec<(Icon, ColoredString)>, ui: &mut Ui) {
        for (icon, value) in lines {
            ui.horizontal(|ui| {
                icon.show(ui);
                value.as_label(ui).wrap(true).ui(ui);
            });
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
        let houses = self.houses(world)?;
        Self::show_houses(houses, ui);
        let rarity = self.rarity();
        if let Ok(rarity) = rarity {
            Self::show_rarity(rarity, ui);
        }
        let expanded = ui.input(|i| i.modifiers.shift) || SettingsData::get(world).expanded_hint;
        let description = self.description(expanded, world);
        let statuses_colored = self.statuses(&statuses, world);
        let extra_lines = self.extra_lines();
        frame(ui, |ui| {
            if let Ok(lines) = description {
                Self::show_description(lines, ui);
            }
            if !expanded {
                if let Ok(statuses) = statuses_colored.as_ref() {
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
                    .add_color(white())
                    .set_style_ref(ColoredStringStyle::Small)
                    .label(ui);
            });
            ui.add_space(2.0);
            return Ok(());
        }
        if let Ok(statuses) = statuses_colored.as_ref() {
            frame(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(RichText::new("Statuses").color(white()));
                });
                Self::show_status_lines(statuses, open, ui);
            });
        }
        let definitions = self.definitions(statuses.as_ref(), world);
        if let Ok(definitions) = definitions {
            for (name, text) in &definitions {
                frame(ui, |ui| {
                    name.label(ui);
                    ui.horizontal_wrapped(|ui| {
                        text.as_label(ui).wrap(true).ui(ui);
                    });
                });
            }
        }

        Ok(())
    }

    pub fn show_state_card_ui(
        &self,
        id: impl std::hash::Hash,
        statuses: Vec<(String, i32)>,
        open: bool,
        ui: &mut Ui,
        world: &World,
    ) {
        Self::window(self, id, open).show_ui(ui, |ui| {
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
        if let Some(visibility) = world.get::<ViewVisibility>(entity) {
            if !visibility.get() {
                return;
            }
        }
        let id = Id::new(entity);
        Self::window(self, id, open)
            .entity_anchor(entity, Align2::CENTER_TOP, vec2(0.0, -1.2), world)
            .show(ctx, |ui| {
                let _ = self.show_frames(statuses, open, ui, world);
            });
    }

    fn window(&self, id: impl std::hash::Hash, open: bool) -> GameWindow<'static> {
        let w = window("UNIT")
            .id(id)
            .set_width(if open { 300.0 } else { 140.0 })
            .title_bar(false)
            .order(if open {
                egui::Order::Foreground
            } else {
                Order::Middle
            });
        if let Ok(c) = self.get_color(VarName::RarityColor) {
            w.set_color(c.c32())
        } else {
            w
        }
    }
}
