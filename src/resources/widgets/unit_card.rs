use super::*;

pub fn unit_card(t: f32, state: &VarState, ui: &mut Ui, world: &World) -> Result<()> {
    let houses = state.get_value_at(VarName::Houses, t)?.get_string_list()?;
    let name = state.get_string_at(VarName::Name, t)?;
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

    let rect = Frame {
        inner_margin: Margin::same(8.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding {
            nw: 13.0,
            ne: 13.0,
            sw: 0.0,
            se: 0.0,
        },
        shadow: Shadow::NONE,
        fill: DARK_BLACK,
        stroke: Stroke {
            width: 1.0,
            color: name_color(&houses[0]),
        },
    }
    .show(ui, |ui| {
        name.cstr_cs(name_color(&houses[0]), CstrStyle::Heading)
            .label(ui);

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
        for house in houses {
            houses_cstr.push(house.cstr_c(name_color(&house)));
        }
        houses_cstr.join(&" + ".cstr()).label(ui);
        ui.add_space(2.0);
    })
    .response
    .rect;
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
        ui.horizontal_wrapped(|ui| {
            for (name, charges) in statuses {
                if name.eq(LOCAL_STATUS) {
                    continue;
                }
                format!("{name} ({charges})")
                    .cstr_c(name_color(&name))
                    .label(ui);
            }
        });
        br(ui);
        ui.horizontal_wrapped(|ui| {
            for name in used_definitions {
                name.cstr_cs(name_color(&name), CstrStyle::Bold).label(ui);
                definition(&name)
                    .inject_ability_defaults(&name, world)
                    .as_label(ui)
                    .wrap(true)
                    .ui(ui);
            }
        });
    });
    ui.painter()
        .circle_filled(rect.center_bottom(), 13.0, LIGHT_BLACK);
    ui.painter().circle_filled(
        rect.center_bottom(),
        10.0,
        state.get_color(VarName::RarityColor)?.c32(),
    );
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
