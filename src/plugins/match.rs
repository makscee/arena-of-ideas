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

impl MatchPlugin {
    pub fn load_match_data(id: u64, world: &mut World) {
        let m = Match::from_table(NodeDomain::Match, id).unwrap();
        let mut team_world = World::new();
        dbg!(&m);
        m.team
            .unwrap()
            .unpack(team_world.spawn_empty().id(), &mut team_world);

        let mut core_world = World::new();
        for house in NodeDomain::Core.filter_by_kind(NodeKind::House) {
            let house = House::from_table(NodeDomain::Core, house.id).unwrap();
            house.unpack(core_world.spawn_empty().id(), &mut core_world);
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
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for unit in md.team_world.query::<&Unit>().iter(&md.team_world) {
                        Frame::none()
                            .stroke(STROKE_DARK)
                            .inner_margin(Margin::same(2.0))
                            .outer_margin(Margin::same(2.0))
                            .rounding(ROUNDING)
                            .show(ui, |ui| {
                                let stats = md.team_world.get::<UnitStats>(unit.entity()).unwrap();
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
                        })
                    },
                )
            });
            for (entity, slot) in md
                .team_world
                .query::<(Entity, &UnitSlot)>()
                .iter(&md.team_world)
            {
                let slot = slot.slot as usize;
            }

            world.insert_resource(md);
        })
        .default_width(800.0)
        .default_height(600.0)
        .push(world);
    }
}
