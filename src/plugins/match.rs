use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource)]
struct MatchData {
    g: i32,
    shop_units: Vec<Option<Unit>>,
    team_units: Vec<Unit>,
}

impl MatchPlugin {
    pub fn load_match_data(id: u64, world: &mut World) {
        let m = Match::from_table(NodeDomain::Match, id).unwrap();
        let team_units = m
            .team
            .unwrap()
            .collect_units()
            .into_iter()
            .cloned()
            .collect_vec();
        world.insert_resource(MatchData {
            g: m.g,
            shop_units: m
                .shop_case
                .into_iter()
                .map(|d| {
                    if !d.sold {
                        let id = d.unit_id;
                        let unit = Unit::from_table(NodeDomain::Alpha, id);
                        if unit.is_none() {
                            error!("Alpha unit#{id} not found");
                        }
                        unit
                    } else {
                        None
                    }
                })
                .collect(),
            team_units,
        });
    }
    fn show_slot(
        unit: Option<&Unit>,
        ui: &mut Ui,
        world: &mut World,
        add_contents: impl FnOnce(&mut Ui),
    ) -> Response {
        const FRAME: Frame = Frame {
            inner_margin: Margin::ZERO,
            outer_margin: Margin::ZERO,
            rounding: Rounding::same(13.0),
            shadow: SHADOW,
            fill: BG_DARK,
            stroke: Stroke {
                width: 1.0,
                color: VISIBLE_LIGHT,
            },
        };
        ui.with_layout(
            Layout::bottom_up(Align::Center).with_cross_justify(true),
            |ui| {
                add_contents(ui);
                let rect = ui.available_rect_before_wrap();
                let size = rect.width().at_most(rect.height());
                let rect = Rect::from_center_size(rect.center(), egui::vec2(size, size));
                let resp = ui.allocate_rect(rect, Sense::click_and_drag());
                if resp.hovered() {
                    ui.painter().rect_stroke(rect, ROUNDING, STROKE_YELLOW);
                } else {
                    ui.painter().rect_stroke(rect, ROUNDING, STROKE_DARK);
                }
                if let Some(unit) = unit {
                    let context = Context::default().set_owner_node(unit).take();
                    if resp.hovered() && !resp.dragged() {
                        cursor_window_frame(ui.ctx(), FRAME, 350.0, |ui| {
                            unit.show(None, &context, ui);
                        });
                    }
                    if resp.clicked() {
                        let unit = unit.clone();
                        Window::new(unit.name.clone(), move |ui, _| {
                            let context = Context::default().set_owner_node(&unit).take();
                            unit.show(None, &context, ui);
                        })
                        .push(world);
                    }
                    let rect = rect.shrink(15.0);
                    let rep = unit.representation.as_ref().unwrap();
                    RepresentationPlugin::paint_rect(rect, &context, &unit_rep().material, ui)
                        .log();
                    RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui).log();
                }
                resp
            },
        )
        .inner
    }
    pub fn open_shop_window(world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        Window::new("Match", move |ui, world| {
            let md = world.remove_resource::<MatchData>().unwrap();
            ui.horizontal(|ui| {
                format!("[yellow [b {}g]]", md.g).label(ui);
                if format!("Reroll [yellow [b {}g]]", global_settings().match_g.reroll)
                    .button(ui)
                    .clicked()
                {
                    cn().reducers.match_reroll().unwrap();
                }
            });
            let shop_units = &md.shop_units;
            let team_units = &md.team_units;
            let full_rect = ui.available_rect_before_wrap();
            let shop_rect = full_rect.with_max_y(full_rect.min.y + full_rect.height() * 0.5);
            let team_rect = full_rect.with_min_y(shop_rect.max.y);
            let shop_ui = &mut ui.child_ui(shop_rect, *ui.layout(), None);
            let team_ui = &mut ui.child_ui(team_rect, *ui.layout(), None);
            shop_ui.columns(shop_units.len(), |ui| {
                for (i, unit) in shop_units.iter().enumerate() {
                    let ui = &mut ui[i];
                    let unit = unit.as_ref();
                    Self::show_slot(unit, ui, world, |ui| {
                        if ui.button("Buy").clicked() {
                            cn().reducers.match_buy(i as u8).unwrap();
                        }
                    });
                }
            });
            team_ui.columns(5, |ui| {
                for (i, unit) in team_units.iter().enumerate() {
                    let ui = &mut ui[i];
                    let resp = Self::show_slot(Some(unit), ui, world, |ui| {
                        if ui.button("Sell").clicked() {
                            cn().reducers.match_sell(i as u8).unwrap();
                        }
                    });
                    if resp.dragged() {
                        let origin = resp.rect.center();
                        if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                            ui.painter().arrow(
                                origin,
                                pointer.to_vec2() - origin.to_vec2(),
                                Stroke::new(3.0, YELLOW),
                            );
                        }
                    }
                    resp.dnd_set_drag_payload(i);
                    if let Some(drop_i) = resp.dnd_release_payload::<usize>() {
                        cn().reducers.match_reorder(*drop_i as u8, i as u8).unwrap();
                    }
                }
            });
            world.insert_resource(md);
        })
        .default_width(500.0)
        .push(world);
    }
}
