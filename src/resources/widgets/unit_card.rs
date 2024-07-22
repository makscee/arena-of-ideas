use super::*;

pub fn unit_card(context: &Context, ui: &mut Ui, world: &World) -> Result<()> {
    let owner = context.owner();
    let houses = context
        .get_value(VarName::Houses, world)?
        .get_string_list()?;
    let house_colors = context
        .get_value(VarName::HouseColors, world)?
        .get_color_list()?
        .into_iter()
        .map(|c| c.c32())
        .collect_vec();
    let name = entity_name(owner).style(CstrStyle::Heading).take();
    let fusible_lvl = houses.len() as i32 + 1;
    let fusible_str = if fusible_lvl > context.get_int(VarName::Lvl, world).unwrap_or_default() {
        "Fusible from lvl "
            .cstr()
            .push(fusible_lvl.to_string().cstr_cs(PURPLE, CstrStyle::Bold))
            .take()
    } else {
        "Fusible".cstr_cs(YELLOW, CstrStyle::Bold)
    };
    let used_definitions = context
        .get_value(VarName::UsedDefinitions, world)?
        .get_string_list()?;
    let triggers = context
        .get_value(VarName::TriggersDescription, world)?
        .get_cstr_list()?;
    let targets = context
        .get_value(VarName::TargetsDescription, world)?
        .get_cstr_list()?;
    let mut effects = context
        .get_value(VarName::EffectsDescription, world)?
        .get_cstr_list()?;
    for c in effects.iter_mut() {
        c.inject_context(context, world);
    }

    let rect = Frame {
        inner_margin: Margin::same(8.0),
        outer_margin: Margin::ZERO,
        rounding: Rounding::ZERO,
        shadow: Shadow::NONE,
        fill: EMPTINESS,
        stroke: Stroke::NONE,
    }
    .show(ui, |ui| {
        name.label(ui);
        const SHOWN_VARS: [(VarName, Color32); 4] = [
            (VarName::Pwr, YELLOW),
            (VarName::Hp, RED),
            (VarName::Lvl, PURPLE),
            (VarName::Xp, LIGHT_PURPLE),
        ];
        ui.horizontal_wrapped(|ui| {
            for (var, color) in SHOWN_VARS.iter().copied() {
                let mut vars_str = var.to_string().cstr_c(color);
                vars_str.push(": ".cstr_c(color));
                vars_str.push(
                    context
                        .get_string(var, world)
                        .unwrap_or_default()
                        .cstr_c(VISIBLE_BRIGHT),
                );
                match var {
                    VarName::Xp => {
                        vars_str.push("/".cstr()).push(
                            context
                                .get_string(VarName::Lvl, world)
                                .unwrap_or_default()
                                .cstr_c(VISIBLE_BRIGHT),
                        );
                    }
                    _ => {}
                }
                vars_str.bold().label(ui);
                ui.add_space(2.0);
            }
        });
        fusible_str.label(ui);

        let mut houses_cstr = Cstr::default();
        for (i, house) in houses.into_iter().enumerate() {
            houses_cstr.push(house.cstr_c(house_colors[i]));
        }
        houses_cstr.join(&" + ".cstr()).label(ui);
        ui.add_space(2.0);
    })
    .response
    .rect;

    if house_colors.len() > 1 {
        let len = house_colors.len() as f32;
        let t = gt().play_head() * 0.1;
        for (i, color) in house_colors.iter().copied().enumerate() {
            let from = (i as f32 / len + t).fract();
            let to = ((i + 1) as f32 / len + t).fract();
            lines_around_rect((from, to), &rect, color, ui);
        }
    } else {
        lines_around_rect((0.0, 1.0), &rect, house_colors[0], ui);
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
        fill: BG_DARK,
        stroke: Stroke::NONE,
    }
    .show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        show_trigger_part("trg:", triggers, EVENT_COLOR, ui);
        show_trigger_part("tar:", targets, TARGET_COLOR, ui);
        show_trigger_part("eff:", effects, EFFECT_COLOR, ui);

        br(ui);
        let statuses = context
            .all_active_statuses(world)
            .into_iter()
            .filter(|(_, c)| *c > 0)
            .collect_vec();
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
                        .inject_ability_state(
                            &name,
                            context.clone().set_ability_state(&name, world).unwrap(),
                        )
                        .as_label(ui)
                        .wrap(true)
                        .ui(ui);
                });
            }
        });
    });
    let rarities = context
        .get_value(VarName::RarityColors, world)?
        .get_color_list()?;
    const OFFSET: egui::Vec2 = egui::vec2(33.0, 0.0);
    let from = rect.center_bottom() - (rarities.len() as f32 - 1.0) * 0.5 * OFFSET;
    for (i, color) in rarities.into_iter().enumerate() {
        let pos = from + OFFSET * i as f32;
        ui.painter().circle_filled(pos, 13.0, BG_LIGHT);
        ui.painter().circle_filled(pos, 10.0, color.c32());
    }
    Ok(())
}

fn show_trigger_part(title: &str, content: Vec<Cstr>, color: Color32, ui: &mut Ui) {
    ui.horizontal(|ui| {
        title.cstr_c(VISIBLE_DARK).label(ui);
        let rect = Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for c in content {
                        c.as_label(ui).wrap(true).ui(ui);
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

fn lines_around_rect(range: (f32, f32), rect: &Rect, color: Color32, ui: &mut Ui) {
    let mut path = vec![point_on_rect(range.0, rect)];
    let w_part = rect.width() / (rect.width() + rect.height()) * 0.5;
    let points = [
        (0.0, rect.left_top()),
        (w_part, rect.right_top()),
        (0.5, rect.right_bottom()),
        (0.5 + w_part, rect.left_bottom()),
        (1.0, rect.left_top()),
    ];
    let mut start = 0;
    let mut end = 0;
    for i in 0..(points.len() - 1) {
        if range.0 >= points[i].0 && range.0 <= points[i + 1].0 {
            start = i + 1;
        }
        if range.1 >= points[i].0 && range.1 <= points[i + 1].0 {
            end = i + 1;
        }
    }
    if start > end {
        end += points.len();
    }
    for i in start..end {
        path.push(points[i % points.len()].1);
    }
    path.push(point_on_rect(range.1, rect));
    ui.painter()
        .add(egui::Shape::line(path, Stroke { width: 1.0, color }));
}

fn point_on_rect(t: f32, rect: &Rect) -> egui::Pos2 {
    let w_part = rect.width() / (rect.width() + rect.height());
    if t < 0.5 {
        let t = t * 2.0;
        if t < w_part {
            let t = t / w_part;
            rect.left_top() + (rect.right_top() - rect.left_top()) * t
        } else {
            let t = (t - w_part) / (1.0 - w_part);
            rect.right_top() + (rect.right_bottom() - rect.right_top()) * t
        }
    } else {
        let t = (t - 0.5) * 2.0;
        if t < w_part {
            let t = t / w_part;
            rect.right_bottom() + (rect.left_bottom() - rect.right_bottom()) * t
        } else {
            let t = (t - w_part) / (1.0 - w_part);
            rect.left_bottom() + (rect.left_top() - rect.left_bottom()) * t
        }
    }
}
