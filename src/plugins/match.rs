use bevy_egui::egui::{emath::GuiRounding, UiBuilder};

use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Match), Self::on_enter);
    }
}

#[derive(Resource)]
struct MatchData {
    g: i32,
    shop_case: Vec<ShopCaseUnit>,
    team_world: World,
    core_world: World,
    editing_entity: Option<Entity>,
}

const FRAME: Frame = Frame {
    inner_margin: Margin::same(5),
    outer_margin: Margin::same(5),
    corner_radius: ROUNDING,
    shadow: Shadow::NONE,
    fill: TRANSPARENT,
    stroke: STROKE_DARK,
};

impl MatchPlugin {
    fn on_enter(world: &mut World) {
        let m = NodeDomain::Match
            .filter_by_kind(NodeKind::Match)
            .into_iter()
            .next();
        if let Some(m) = m {
            if m.owner != player_id() {
                "Tried to open match not owned by current player".notify_error(world);
                GameState::Title.set_next(world);
                return;
            }
            Self::load_match_data(m.id, world);
        } else {
            "No active match found".notify_error(world);
            GameState::Title.set_next(world);
        }
    }
    pub fn load_match_data(id: u64, world: &mut World) {
        let m = Match::from_table(NodeDomain::Match, id).unwrap();
        let mut team_world = World::new();
        let mut core_world = World::new();
        for house in NodeDomain::Core.filter_by_kind(NodeKind::House) {
            let house = House::from_table(NodeDomain::Core, house.id).unwrap();
            house.unpack(core_world.spawn_empty().id(), &mut core_world);
        }
        m.team
            .unwrap()
            .unpack(team_world.spawn_empty().id(), &mut team_world);

        let shop_case = m.shop_case;
        world.insert_resource(MatchData {
            g: m.g,
            shop_case,
            team_world,
            core_world,
            editing_entity: None,
        });
    }
    pub fn shop_tab(ui: &mut Ui, world: &mut World) {
        world.resource_scope(|_, d: Mut<MatchData>| {
            ui.horizontal(|ui| {
                format!("[yellow [h2 {}g]]", d.g).label(ui);
                if format!("[h2 reroll [yellow {}g]]", global_settings().match_g.reroll)
                    .button(ui)
                    .clicked()
                {
                    cn().reducers.match_reroll().unwrap();
                }
            });
            let shop_slots = d.shop_case.len();
            let full_rect = ui.available_rect_before_wrap();
            let slot_rect =
                full_rect.with_max_x(full_rect.left() + full_rect.width() / shop_slots as f32);
            ui.horizontal(|ui| {
                for i in 0..shop_slots {
                    let rect = slot_rect.translate(egui::vec2(i as f32 * slot_rect.width(), 0.0));
                    ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
                        Self::show_shop_slot(&d, i, ui);
                    });
                }
            });
        });
    }
    pub fn roster_tab(ui: &mut Ui, world: &mut World) {
        world.resource_scope(|_, mut d: Mut<MatchData>| {
            for (unit, stats) in d
                .team_world
                .query::<(&Unit, &UnitStats)>()
                .iter(&d.team_world)
            {
                FRAME.show(ui, |ui| {
                    show_unit_tag(unit, stats, ui, &d.team_world);
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
        });
    }
    pub fn team_tab(ui: &mut Ui, world: &mut World) {
        world.resource_scope(|_, mut d: Mut<MatchData>| {
            let mut fusion_edit = None;
            let mut last_slot = -1;
            for (fusion, slot, rep) in d
                .team_world
                .query::<(&Fusion, &UnitSlot, &Representation)>()
                .iter(&d.team_world)
            {
                let slot = slot.slot;
                let r = show_slot(
                    slot as usize,
                    global_settings().team_slots as usize,
                    false,
                    ui,
                );
                last_slot = last_slot.at_least(slot);
                fusion.paint(r.rect, ui, &d.team_world).unwrap();
                let context = &Context::new_world(&d.team_world)
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
                    if slot as usize != *i {
                        cn().reducers.match_reorder(*i as u8, slot as u8).unwrap();
                    }
                }
            }
            if last_slot + 1 < global_settings().team_slots as i32 {
                let slot = last_slot + 1;
                let r = show_slot(
                    slot as usize,
                    global_settings().team_slots as usize,
                    false,
                    ui,
                );
                if r.clicked() {
                    fusion_edit = Some(slot);
                }
            }
        });
    }

    fn show_shop_slot(md: &MatchData, i: usize, ui: &mut Ui) {
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
                TagWidget::new_text(name, if sc.sold { VISIBLE_DARK } else { color }).ui(ui);
                let size = ui.available_size();
                let size = size.x.at_most(size.y);
                let rect = ui
                    .allocate_new_ui(
                        UiBuilder::new().max_rect(
                            Rect::from_center_size(
                                ui.available_rect_before_wrap().center(),
                                egui::vec2(size, size),
                            )
                            .round_ui(),
                        ),
                        |ui| show_slot(i, 1, false, ui).rect,
                    )
                    .inner
                    .shrink(10.0);
                if !sc.sold {
                    RepresentationPlugin::paint_rect(rect, context, &unit_rep().material, ui).log();
                    if let Some(rep) = md.core_world.get::<Representation>(entity) {
                        RepresentationPlugin::paint_rect(rect, context, &rep.material, ui).log();
                    }
                }
            },
        );
    }
    fn show_shop(ui: &mut Ui, world: &mut World) {
        let mut md = world.remove_resource::<MatchData>().unwrap();
        let full_rect = ui.available_rect_before_wrap();
        let mut fusion_edit = None;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    format!("[yellow [h2 {}g]]", md.g).label(ui);
                    if format!("[h2 reroll [yellow {}g]]", global_settings().match_g.reroll)
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
            ui.allocate_new_ui(
                UiBuilder::new().max_rect(rect.with_min_y(rect.top() + rect.height() * 0.5 + 5.0)),
                |ui| {
                    let mut last_slot = -1;
                    for (fusion, slot, rep) in md
                        .team_world
                        .query::<(&Fusion, &UnitSlot, &Representation)>()
                        .iter(&md.team_world)
                    {
                        let slot = slot.slot;
                        let r = show_slot(
                            slot as usize,
                            global_settings().team_slots as usize,
                            false,
                            ui,
                        );
                        last_slot = last_slot.at_least(slot);
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
                            if slot as usize != *i {
                                cn().reducers.match_reorder(*i as u8, slot as u8).unwrap();
                            }
                        }
                    }
                    if last_slot + 1 < global_settings().team_slots as i32 {
                        let slot = last_slot + 1;
                        let r = show_slot(
                            slot as usize,
                            global_settings().team_slots as usize,
                            false,
                            ui,
                        );
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
    }
    fn open_fusion_edit_window(slot: i32, world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        let mut md = world.remove_resource::<MatchData>().unwrap();
        let entity = if let Some(fusion) = Fusion::find_by_slot(slot, &mut md.team_world) {
            fusion.entity()
        } else {
            let team = md
                .team_world
                .query::<&Team>()
                .single(&md.team_world)
                .entity();
            let entity = md.team_world.spawn_empty().set_parent(team).id();
            Fusion::new_full(default(), default(), UnitSlot::new(slot))
                .unpack(entity, &mut md.team_world);
            entity
        };
        md.editing_entity = Some(entity);
        Fusion::open_editor_window(entity, world, &md.team_world, |f, world| {
            let mut md = world.resource_mut::<MatchData>();
            let entity = md.editing_entity.unwrap();
            f.unpack(entity, &mut md.team_world);
            let fusions = md
                .team_world
                .query::<&Fusion>()
                .iter(&md.team_world)
                .map(|f| f.to_strings_root())
                .collect_vec();
            cn().reducers.match_edit_fusions(fusions).unwrap();
        })
        .log();
        world.insert_resource(md);
    }
}
