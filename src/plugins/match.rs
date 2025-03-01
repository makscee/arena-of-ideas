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
        let player = Player::get(player_entity(), world).unwrap();
        let Ok(m) = player.active_match_load(world) else {
            "No active match found".notify_error(world);
            GameState::Title.set_next(world);
            return;
        };
    }
    pub fn shop_tab(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let player = player(world)?;
        let m = player.active_match_load(world)?;

        ui.horizontal(|ui| {
            format!("[yellow [h2 {}g]]", m.g).label(ui);
            if format!("[h2 reroll [yellow {}g]]", global_settings().match_g.reroll)
                .button(ui)
                .clicked()
            {
                cn().reducers.match_reroll().unwrap();
            }
        });
        let shop_case = m.shop_case_load(world)?;
        let shop_slots = shop_case.len();
        let full_rect = ui.available_rect_before_wrap();
        let slot_rect =
            full_rect.with_max_x(full_rect.left() + full_rect.width() / shop_slots as f32);
        ui.horizontal(|ui| {
            for (i, sc) in shop_case.into_iter().enumerate() {
                let rect = slot_rect.translate(egui::vec2(i as f32 * slot_rect.width(), 0.0));
                ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
                    Self::show_shop_slot(sc, i, ui, world)
                })
                .inner?;
            }
            Ok(())
        })
        .inner
    }
    pub fn roster_tab(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let player = player(world)?;
        let m = player.active_match_load(world)?;
        let Ok(houses) = m.team_load(world)?.houses_load(world) else {
            return Ok(());
        };
        for unit in houses
            .into_iter()
            .filter_map(|h| h.units_load(world).ok())
            .flatten()
        {
            let stats = unit.description_load(world)?.stats_load(world)?;
            DARK_FRAME.show(ui, |ui| {
                show_unit_tag(unit, stats, ui, world);
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
        Ok(())
    }
    pub fn team_tab(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let player = player(world)?;
        let m = player.active_match_load(world)?;

        let mut fusion_edit = None;
        let mut last_slot = -1;
        let Ok(fusions) = m.team_load(world)?.fusions_load(world) else {
            return Ok(());
        };
        for fusion in fusions {
            let slot = fusion.slot;
            let r = show_slot(
                slot as usize,
                global_settings().team_slots as usize,
                false,
                ui,
            );
            last_slot = last_slot.at_least(slot);
            fusion.paint(r.rect, ui, world).unwrap();
            let context = &Context::new_world(world).set_owner(fusion.entity()).take();
            for rep in fusion.collect_children::<Representation>(world) {
                rep.paint(r.rect, context, ui).log();
            }
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

        if let Some(slot) = fusion_edit {
            Self::edit_fusion(slot, world);
        }
        Ok(())
    }

    fn show_shop_slot(
        sc: &ShopCaseUnit,
        i: usize,
        ui: &mut Ui,
        world: &World,
    ) -> Result<(), ExpressionError> {
        ui.with_layout(
            Layout::bottom_up(Align::Center).with_cross_justify(true),
            |ui| {
                if format!("[b buy [yellow {}g]]", sc.price)
                    .as_button()
                    .enabled(!sc.sold)
                    .ui(ui)
                    .clicked()
                {
                    cn().reducers.match_buy(i as u8).unwrap();
                }
                let entity = core_unit_by_name(&sc.unit)?;
                let context = &Context::new_world(&world).set_owner(entity).take();
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
                    if let Some(rep) = world.get::<Representation>(entity) {
                        RepresentationPlugin::paint_rect(rect, context, &rep.material, ui).log();
                    }
                }
                Ok(())
            },
        )
        .inner
    }
    fn edit_fusion(slot: i32, world: &mut World) {
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
            Fusion::new_full(slot, default(), default()).unpack(entity, &mut md.team_world);
            entity
        };
        md.editing_entity = Some(entity);
        FusionEditorPlugin::edit_entity(entity, world, &md.team_world, |f, world| {
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
            GameState::Match.set_next(world);
        })
        .log();
        world.insert_resource(md);
    }
}
