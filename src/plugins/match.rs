use bevy::ecs::world::FromWorld;

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
    fn show_unit_tag(md: &MatchData, unit: &Unit, stats: &UnitStats, ui: &mut Ui) {
        TagWidget::new_number(
            &unit.name,
            Context::new_world(&md.team_world)
                .set_owner(unit.entity())
                .get_color(VarName::color)
                .unwrap(),
            format!(
                "[b {} {}]",
                stats.pwr.cstr_c(VarName::pwr.color()),
                stats.hp.cstr_c(VarName::hp.color())
            ),
        )
        .ui(ui);
    }
    pub fn open_shop_window(world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        Window::new("Match", move |ui, world| {
            let mut md = world.remove_resource::<MatchData>().unwrap();
            let full_rect = ui.available_rect_before_wrap();
            let mut edit_requested = false;
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for (unit, stats) in md
                        .team_world
                        .query::<(&Unit, &UnitStats)>()
                        .iter(&md.team_world)
                    {
                        FRAME.show(ui, |ui| {
                            Self::show_unit_tag(&md, unit, stats, ui);
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
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                format!("[yellow [h2 {}g]]", md.g).label(ui);
                                if format!(
                                    "[b reroll [yellow {}g]]",
                                    global_settings().match_g.reroll
                                )
                                .button(ui)
                                .clicked()
                                {
                                    cn().reducers.match_reroll().unwrap();
                                }
                            });
                            if "edit".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                edit_requested = true;
                            }
                        });
                        for (fusion, slot, rep) in md
                            .team_world
                            .query::<(&Fusion, &UnitSlot, &Representation)>()
                            .iter(&md.team_world)
                        {
                            let r = show_slot(
                                slot.slot as usize,
                                global_settings().team_slots as usize,
                                false,
                                ui,
                            );
                            fusion.paint(r.rect, ui, &md.team_world).unwrap();
                            let context = &Context::new_world(&md.team_world)
                                .set_owner(fusion.entity())
                                .take();
                            RepresentationPlugin::paint_rect(r.rect, context, &rep.material, ui)
                                .log();
                        }
                    },
                )
            });
            world.insert_resource(md);
            if edit_requested {
                Self::open_fusion_edit_window(0, world);
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
        let mut fusion = Fusion::default();
        let window_id = "Fusion Edit";
        Window::new(window_id, move |ui, world| {
            let mut md = world.remove_resource::<MatchData>().unwrap();
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
                                Self::show_unit_tag(&md, unit, stats, ui);
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
                            let trigger_ref = UnitTriggerRef {
                                unit: u as u8,
                                trigger: t as u8,
                            };
                            let selected = fusion
                                .triggers
                                .iter()
                                .find_position(|(r, _)| r.eq(&trigger_ref))
                                .map(|(i, _)| i);
                            FRAME
                                .stroke(if selected.is_some() {
                                    STROKE_YELLOW
                                } else {
                                    STROKE_DARK
                                })
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        trigger.show(None, context, ui);
                                        if ">".cstr_s(CstrStyle::Bold).button(ui).clicked() {
                                            if let Some(ind) = selected {
                                                fusion.triggers.remove(ind);
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
                                let action_ref = UnitActionRef {
                                    unit: u as u8,
                                    trigger: t as u8,
                                    action: a as u8,
                                };
                                let selected = fusion
                                    .triggers
                                    .iter()
                                    .any(|(_, a)| a.iter().any(|a| action_ref.eq(a)));
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
                                                    for (_, actions) in &mut fusion.triggers {
                                                        if let Some((action, _)) = actions
                                                            .iter()
                                                            .find_position(|a| action_ref.eq(a))
                                                        {
                                                            actions.remove(action);
                                                            break;
                                                        }
                                                    }
                                                } else {
                                                    fusion
                                                        .triggers
                                                        .last_mut()
                                                        .unwrap()
                                                        .1
                                                        .push(action_ref);
                                                }
                                            }
                                        })
                                    });
                            }
                        }
                    }
                });
                ui.vertical(|ui| {
                    "Result".cstr_s(CstrStyle::Heading2).label(ui);
                    for (trigger, actions) in &fusion.triggers {
                        let trigger = fusion
                            .get_trigger(trigger.unit, trigger.trigger, context)
                            .unwrap();
                        trigger.show(None, context, ui);
                        FRAME.show(ui, |ui| {
                            for action in actions {
                                let (entity, action) = fusion.get_action(action, context).unwrap();
                                action.show(None, context.clone().set_owner(entity), ui);
                            }
                        });
                    }
                    if "save"
                        .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2)
                        .button(ui)
                        .clicked()
                    {
                        cn().reducers
                            .match_edit_fusions([fusion.to_strings_root()].to_vec())
                            .unwrap();
                        WindowPlugin::close_current(world);
                    }
                });
            });
            world.insert_resource(md);
        })
        .push(world);
    }
}
