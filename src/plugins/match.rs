use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, _: &mut App) {}
}

#[derive(Resource)]
struct MatchData {
    g: i32,
    shop_case: Vec<ShopCaseUnit>,
    team_world: World,
    core_world: World,
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(5.0),
    outer_margin: Margin::same(5.0),
    rounding: ROUNDING,
    shadow: Shadow::NONE,
    fill: TRANSPARENT,
    stroke: STROKE_DARK,
};

impl MatchPlugin {
    pub fn load_match_data(id: u64, world: &mut World) {
        let m = Match::from_table(NodeDomain::Match, id).unwrap();
        let mut team_world = World::new();
        dbg!(&m);

        let mut core_world = World::new();
        for house in NodeDomain::Core.filter_by_kind(NodeKind::House) {
            let house = House::from_table(NodeDomain::Core, house.id).unwrap();
            house.unpack(core_world.spawn_empty().id(), &mut core_world);
        }
        m.team
            .unwrap()
            .unpack(team_world.spawn_empty().id(), &mut team_world);
        for fusion in team_world
            .query::<&Fusion>()
            .iter(&team_world)
            .cloned()
            .collect_vec()
        {
            fusion.init(&mut team_world).unwrap();
        }

        let shop_case = m.shop_case;
        world.insert_resource(MatchData {
            g: m.g,
            shop_case,
            team_world,
            core_world,
        });
    }
    fn show_shop_case(md: &mut MatchData, ui: &mut Ui) {
        let shop_slots = md.shop_case.len();
        ui.columns(shop_slots, |ui| {
            for i in 0..shop_slots {
                let ui = &mut ui[i];
                ui.with_layout(
                    Layout::bottom_up(Align::Center).with_cross_justify(true),
                    |ui| {
                        let sc = &md.shop_case[i];
                        let Some(entity) = md.core_world.get_id_link(sc.unit_id) else {
                            return;
                        };

                        if format!("[b buy [yellow {}g]]", global_settings().match_g.unit_buy)
                            .as_button()
                            .enabled(!sc.sold)
                            .ui(ui)
                            .clicked()
                        {
                            cn().reducers.match_buy(i as u8).unwrap();
                        }
                        let context = &Context::new_world(&md.core_world).set_owner(entity).take();
                        let name = context.get_string(VarName::name).unwrap();
                        let color = context.get_color(VarName::color).unwrap();
                        TagWidget::new_text(name, if sc.sold { VISIBLE_DARK } else { color })
                            .ui(ui);
                        let size = ui.available_size();
                        let size = size.x.at_most(size.y);
                        let rect = ui
                            .allocate_ui_at_rect(
                                Rect::from_center_size(
                                    ui.available_rect_before_wrap().center(),
                                    egui::vec2(size, size),
                                ),
                                |ui| show_slot(i, 1, false, ui).rect,
                            )
                            .inner
                            .shrink(10.0);
                        if !sc.sold {
                            RepresentationPlugin::paint_rect(
                                rect,
                                context,
                                &unit_rep().material,
                                ui,
                            )
                            .log();
                            if let Some(rep) = md.core_world.get::<Representation>(entity) {
                                RepresentationPlugin::paint_rect(rect, context, &rep.material, ui)
                                    .log();
                            }
                        }
                    },
                );
            }
        });
    }
    pub fn open_shop_window(world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        Window::new("Match", move |ui, world| {
            let mut md = world.remove_resource::<MatchData>().unwrap();
            let full_rect = ui.available_rect_before_wrap();
            let mut fusion_edit = None;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        format!("[yellow [h2 {}g]]", md.g).label(ui);
                        if format!("[b reroll [yellow {}g]]", global_settings().match_g.reroll)
                            .button(ui)
                            .clicked()
                        {
                            cn().reducers.match_reroll().unwrap();
                        }
                    });
                    for (unit, stats) in md
                        .team_world
                        .query::<(&Unit, &UnitStats)>()
                        .iter(&md.team_world)
                    {
                        FRAME.show(ui, |ui| {
                            show_unit_tag(unit, stats, ui, &md.team_world);
                            if format!(
                                "[b sell [yellow +{}g]]",
                                global_settings().match_g.unit_sell
                            )
                            .button(ui)
                            .clicked()
                            {
                                cn().reducers.match_sell(unit.name.clone()).unwrap();
                            }
                        });
                    }
                    ui.expand_to_include_y(full_rect.bottom());
                });
                let rect = ui.available_rect_before_wrap();
                ui.allocate_ui_at_rect(
                    rect.with_max_y(rect.bottom() - rect.height() * 0.5),
                    |ui| {
                        Self::show_shop_case(&mut md, ui);
                    },
                );
                ui.allocate_ui_at_rect(
                    rect.with_min_y(rect.top() + rect.height() * 0.5 + 5.0),
                    |ui| {
                        let mut last_slot = -1;
                        for (fusion, slot, rep) in md
                            .team_world
                            .query::<(&Fusion, &UnitSlot, &Representation)>()
                            .iter(&md.team_world)
                        {
                            let slot = slot.slot as usize;
                            let r =
                                show_slot(slot, global_settings().team_slots as usize, false, ui);
                            last_slot = last_slot.at_least(slot as i32);
                            fusion.paint(r.rect, ui, &md.team_world).unwrap();
                            let context = &Context::new_world(&md.team_world)
                                .set_owner(fusion.entity())
                                .take();
                            rep.paint(r.rect, context, ui).log();
                            if r.clicked() {
                                fusion_edit = Some(slot);
                            }
                            if r.drag_started() {
                                r.dnd_set_drag_payload(slot);
                            }
                            if r.dragged() {
                                ui.painter().arrow(
                                    r.rect.center(),
                                    ui.ctx().pointer_latest_pos().unwrap_or_default().to_vec2()
                                        - r.rect.center().to_vec2(),
                                    Stroke::new(2.0, YELLOW),
                                );
                            }
                            if let Some(i) = r.dnd_release_payload::<usize>() {
                                if slot != *i {
                                    cn().reducers.match_reorder(*i as u8, slot as u8).unwrap();
                                }
                            }
                        }
                        if last_slot + 1 < global_settings().team_slots as i32 {
                            let slot = (last_slot + 1) as usize;
                            let r =
                                show_slot(slot, global_settings().team_slots as usize, false, ui);
                            if r.clicked() {
                                fusion_edit = Some(slot);
                            }
                        }
                    },
                )
            });
            world.insert_resource(md);
            if let Some(slot) = fusion_edit {
                Self::open_fusion_edit_window(slot, world);
            }
        })
        .default_width(800.0)
        .default_height(600.0)
        .push(world);
    }
    fn open_fusion_edit_window(slot: usize, world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        let mut md = world.resource_mut::<MatchData>();
        let mut fusions = md
            .team_world
            .query::<(&Fusion, &UnitSlot)>()
            .iter(&md.team_world)
            .sort_by_key::<&UnitSlot, _>(|s| s.slot)
            .map(|(f, _)| f.clone())
            .collect_vec();
        if slot >= fusions.len() {
            fusions.push(default());
        }
        let mut fusion = fusions.remove(slot);
        let window_id = "Fusion Edit";
        Window::new(window_id, move |ui, world| {
            let mut md = world.remove_resource::<MatchData>().unwrap();
            let mut init_fusion = false;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    "Select Units"
                        .cstr_cs(VISIBLE_DARK, CstrStyle::Heading2)
                        .label(ui);
                    for (unit, stats) in md
                        .team_world
                        .query::<(&Unit, &UnitStats)>()
                        .iter(&md.team_world)
                    {
                        let selected = fusion.units.contains(&unit.name);
                        FRAME
                            .stroke(if selected { STROKE_YELLOW } else { STROKE_DARK })
                            .show(ui, |ui| {
                                show_unit_tag(unit, stats, ui, &md.team_world);
                                if "select".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                    if selected {
                                        let i = fusion
                                            .units
                                            .iter()
                                            .position(|u| unit.name.eq(u))
                                            .unwrap();
                                        fusion.remove_unit(i as u8);
                                    } else {
                                        fusion.units.push(unit.name.clone());
                                    }
                                    init_fusion = true;
                                }
                            });
                    }
                });
                let context = &Context::new_world(&md.team_world);
                ui.vertical(|ui| {
                    "Select Triggers".cstr_s(CstrStyle::Heading2).label(ui);
                    for u in 0..fusion.units.len() {
                        let triggers = &fusion.get_reaction(u as u8, context).unwrap().triggers;
                        for (t, (trigger, _)) in triggers.iter().enumerate() {
                            let t_ref = UnitTriggerRef {
                                unit: u as u8,
                                trigger: t as u8,
                            };
                            let selected = fusion.triggers.iter().any(|(r, _)| r.eq(&t_ref));
                            FRAME
                                .stroke(if selected { STROKE_YELLOW } else { STROKE_DARK })
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        trigger.show(None, context, ui);
                                        if ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                            if selected {
                                                fusion.remove_trigger(t_ref);
                                            } else {
                                                fusion.triggers.push((
                                                    UnitTriggerRef {
                                                        unit: u as u8,
                                                        trigger: t as u8,
                                                    },
                                                    default(),
                                                ));
                                            }
                                        }
                                    })
                                });
                        }
                    }
                });
                ui.vertical(|ui| {
                    if fusion.triggers.is_empty() {
                        return;
                    }
                    "Select Actions".cstr_s(CstrStyle::Heading2).label(ui);
                    for u in 0..fusion.units.len() {
                        let reaction = &fusion.get_reaction(u as u8, context).unwrap();
                        let triggers = &reaction.triggers;
                        let entity = reaction.entity();
                        for (t, (_, actions)) in triggers.iter().enumerate() {
                            for (a, action) in actions.0.iter().enumerate() {
                                let a_ref = UnitActionRef {
                                    unit: u as u8,
                                    trigger: t as u8,
                                    action: a as u8,
                                };
                                let selected = fusion
                                    .triggers
                                    .iter()
                                    .any(|(_, a)| a.iter().any(|a| a_ref.eq(a)));
                                FRAME
                                    .stroke(if selected { STROKE_YELLOW } else { STROKE_DARK })
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            action.show(
                                                None,
                                                context.clone().set_owner(entity),
                                                ui,
                                            );
                                            if ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                                if selected {
                                                    fusion.remove_action(a_ref);
                                                } else {
                                                    fusion
                                                        .triggers
                                                        .last_mut()
                                                        .unwrap()
                                                        .1
                                                        .push(a_ref);
                                                }
                                            }
                                        })
                                    });
                            }
                        }
                    }
                });
            });
            FRAME.show(ui, |ui| {
                ui.horizontal(|ui| {
                    let context = &Context::new_world(&md.team_world)
                        .set_owner(fusion.entity())
                        .take();
                    ui.vertical(|ui| {
                        "Result".cstr_s(CstrStyle::Heading2).label(ui);
                        let mut remove_t = None;
                        let mut remove_a = None;
                        let mut swap = None;
                        for (t_i, (t_ref, actions)) in fusion.triggers.iter().enumerate() {
                            let trigger = fusion
                                .get_trigger(t_ref.unit, t_ref.trigger, context)
                                .unwrap();
                            ui.horizontal(|ui| {
                                if "<".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                    remove_t = Some(*t_ref);
                                }
                                trigger.show(None, context, ui);
                            });
                            FRAME.show(ui, |ui| {
                                for (a_i, a_ref) in actions.iter().enumerate() {
                                    let (entity, action) =
                                        fusion.get_action(a_ref, context).unwrap();
                                    ui.horizontal(|ui| {
                                        if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
                                            remove_a = Some(*a_ref);
                                        }
                                        if (t_i > 0 || a_i > 0)
                                            && "^"
                                                .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
                                                .button(ui)
                                                .clicked()
                                        {
                                            if a_i == 0 {
                                                swap = Some((
                                                    (t_i, a_i),
                                                    (t_i - 1, fusion.triggers[t_i - 1].1.len()),
                                                ));
                                            } else {
                                                swap = Some(((t_i, a_i), (t_i, a_i - 1)));
                                            }
                                        }
                                        if (t_i + 1 < fusion.triggers.len()
                                            || a_i + 1 < actions.len())
                                            && "v"
                                                .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
                                                .button(ui)
                                                .clicked()
                                        {
                                            if a_i == actions.len() - 1 {
                                                swap = Some(((t_i, a_i), (t_i + 1, 0)));
                                            } else {
                                                swap = Some(((t_i, a_i), (t_i, a_i + 1)));
                                            }
                                        }
                                        action.show(None, context.clone().set_owner(entity), ui);
                                    });
                                }
                            });
                        }
                        if let Some(((from_t, from_a), (to_t, to_a))) = swap {
                            let action = fusion.triggers[from_t].1.remove(from_a);
                            fusion.triggers[to_t].1.insert(to_a, action);
                        }
                        if let Some(r) = remove_a {
                            fusion.remove_action(r);
                        }
                        if let Some(r) = remove_t {
                            fusion.remove_trigger(r);
                        }
                        if "save"
                            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2)
                            .button(ui)
                            .clicked()
                        {
                            fusions.insert(slot, fusion.clone());
                            let fusions = fusions.iter().map(|f| f.to_strings_root()).collect_vec();
                            cn().reducers.match_edit_fusions(fusions).unwrap();
                            WindowPlugin::close_current(world);
                        }
                    });

                    let size = ui.available_size();
                    let size = size.x.at_most(size.y).at_least(150.0);
                    let rect = ui
                        .allocate_exact_size(egui::vec2(size, size), Sense::hover())
                        .0;
                    fusion.paint(rect, ui, &md.team_world).log();
                    unit_rep().paint(rect.shrink(15.0), context, ui).log();
                });
            });
            if init_fusion {
                fusion.clone().init(&mut md.team_world).log();
            }
            world.insert_resource(md);
        })
        .push(world);
    }
}
