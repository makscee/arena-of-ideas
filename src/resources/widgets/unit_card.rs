use super::*;

pub fn unit_card(t: f32, state: &VarState, ui: &mut Ui, world: &World) -> Result<()> {
    let houses = state.get_value_at(VarName::Houses, t)?.get_string_list()?;
    let house_colors = state
        .get_value_at(VarName::HouseColors, t)?
        .get_color_list()?
        .into_iter()
        .map(|c| c.c32())
        .collect_vec();
    let names = state.get_string_at(VarName::Name, t)?;
    let names = names.split("+").collect_vec();
    let used_definitions = state
        .get_value_at(VarName::UsedDefinitions, t)?
        .get_string_list()?;
    let triggers = state
        .get_value_at(VarName::TriggersDescription, t)?
        .get_cstr_list()?;
    let targets = state
        .get_value_at(VarName::TargetsDescription, t)?
        .get_cstr_list()?;
    let effects = state
        .get_value_at(VarName::EffectsDescription, t)?
        .get_cstr_list()?;
    let faction = TeamPlugin::unit_faction(state.entity().unwrap(), world);

    let rect = Frame {
        inner_margin: Margin::same(8.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding::ZERO,
        shadow: Shadow::NONE,
        fill: DARK_BLACK,
        stroke: Stroke::NONE,
    }
    .show(ui, |ui| {
        let mut name = Cstr::default();
        let part = 1.0 / houses.len() as f32;
        for (i, c) in house_colors.iter().enumerate() {
            let n = names[i];
            if i == 0 {
                let n = n.split_at((n.len() as f32 * part).ceil() as usize).0;
                name.push(n.cstr_c(*c));
            } else if i == houses.len() - 1 {
                let n = n
                    .split_at((n.len() as f32 * (1.0 - part)).floor() as usize)
                    .1;
                name.push(n.cstr_c(*c));
            } else {
                let part = (n.len() as f32 * (1.0 - part) * 0.5).ceil() as usize;
                let n = n.split_at(part).1;
                let n = n.split_at(n.len() - part).0;
                name.push(n.cstr_c(*c));
            }
        }
        name.style(CstrStyle::Heading)
            .as_label(ui)
            .wrap(true)
            .ui(ui);

        const SHOWN_VARS: [(VarName, Color32); 4] = [
            (VarName::Pwr, YELLOW),
            (VarName::Hp, RED),
            (VarName::Lvl, PURPLE),
            (VarName::Stacks, LIGHT_PURPLE),
        ];
        ui.horizontal_wrapped(|ui| {
            for (var, color) in SHOWN_VARS.iter().copied() {
                let mut vars_str = var.to_string().cstr_c(color);
                vars_str.push(": ".cstr_c(color));
                vars_str.push(
                    state
                        .get_value_at(var, t)
                        .unwrap_or_default()
                        .get_string()
                        .unwrap_or_default()
                        .cstr_c(WHITE),
                );
                vars_str.bold().label(ui);
                ui.add_space(2.0);
            }
        });

        let mut houses_cstr = Cstr::default();
        for (i, house) in houses.into_iter().enumerate() {
            houses_cstr.push(house.cstr_c(house_colors[i]));
        }
        houses_cstr.join(&" + ".cstr()).label(ui);
        ui.add_space(2.0);
    })
    .response
    .rect;

    ui.painter().line_segment(
        [rect.left_bottom(), rect.left_top()],
        Stroke {
            width: 1.0,
            color: house_colors[0],
        },
    );
    ui.painter().line_segment(
        [rect.right_bottom(), rect.right_top()],
        Stroke {
            width: 1.0,
            color: house_colors[house_colors.len() - 1],
        },
    );
    let from_top = rect.left_top();
    let from_bottom = rect.left_bottom();
    let offset = egui::vec2((rect.width() / house_colors.len() as f32).round(), 0.0);
    for (i, color) in house_colors.iter().copied().enumerate() {
        ui.painter().line_segment(
            [
                from_top + offset * i as f32,
                from_top + offset * (i + 1) as f32,
            ],
            Stroke { width: 1.5, color },
        );
        ui.painter().line_segment(
            [
                from_bottom + offset * i as f32,
                from_bottom + offset * (i + 1) as f32,
            ],
            Stroke { width: 1.0, color },
        );
    }

    ui.add_space(-ui.style().spacing.item_spacing.y + 0.5);
    Frame {
        inner_margin: Margin::same(8.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding {
            nw: 0.0,
            ne: 0.0,
            sw: 13.0,
            se: 13.0,
        },
        shadow: Shadow::NONE,
        fill: LIGHT_BLACK,
        stroke: Stroke::NONE,
    }
    .show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        show_trigger_part("trg:", triggers, EVENT_COLOR, ui);
        show_trigger_part("tar:", targets, TARGET_COLOR, ui);
        show_trigger_part("eff:", effects, EFFECT_COLOR, ui);

        br(ui);
        let statuses = state.all_statuses_at(t);
        if !statuses.is_empty() {
            ui.horizontal_wrapped(|ui| {
                for (name, charges) in statuses {
                    format!("{name} ({charges})")
                        .cstr_c(name_color(&name))
                        .label(ui);
                }
            });
            br(ui);
        }
        ui.vertical_centered_justified(|ui| {
            for name in used_definitions {
                ui.horizontal_wrapped(|ui| {
                    name.cstr_cs(name_color(&name), CstrStyle::Bold).label(ui);
                    definition(&name)
                        .inject_ability_state(&name, faction, t, world)
                        .as_label(ui)
                        .wrap(true)
                        .ui(ui);
                });
            }
        });
    });
    let rarities = state
        .get_value_at(VarName::RarityColors, t)?
        .get_color_list()?;
    const OFFSET: egui::Vec2 = egui::vec2(33.0, 0.0);
    let from = rect.center_bottom() - (rarities.len() as f32 - 1.0) * 0.5 * OFFSET;
    for (i, color) in rarities.into_iter().enumerate() {
        let pos = from + OFFSET * i as f32;
        ui.painter().circle_filled(pos, 13.0, LIGHT_BLACK);
        ui.painter().circle_filled(pos, 10.0, color.c32());
    }
    Ok(())
}

fn show_trigger_part(title: &str, content: Vec<Cstr>, color: Color32, ui: &mut Ui) {
    ui.horizontal(|ui| {
        title.cstr_c(LIGHT_GRAY).label(ui);
        let rect = Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for c in content {
                        c.label(ui);
                    }
                })
            })
            .response
            .rect;
        ui.painter().line_segment(
            [rect.left_top(), rect.left_bottom()],
            Stroke { width: 1.0, color },
        );
    });
}
