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
    stroke: STROKE_BG_DARK,
};

impl MatchPlugin {
    fn on_enter(world: &mut World) {}
    fn load_match(world: &World) -> Result<&Match, ExpressionError> {
        let player = player(world)?;
        let matches = player.active_match_load(world);
        matches.into_iter().next().to_e("Match not found")
    }
    pub fn tab_shop(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let m = Self::load_match(world)?;
        ui.horizontal(|ui| {
            format!("[yellow [h2 {}g]]", m.g).label(ui);
            if format!("[h2 reroll [yellow {}g]]", global_settings().match_g.reroll)
                .button(ui)
                .clicked()
            {
                cn().reducers.match_reroll().unwrap();
            }
        });
        let shop_case = m.shop_case_load(world);
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
    pub fn tab_roster(ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let m = Self::load_match(world)?;
        let houses = m.team_load(world)?.houses_load(world);
        for unit in houses.into_iter().map(|h| h.units_load(world)).flatten() {
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
    pub fn tab_team(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let m = Self::load_match(world)?;
        let mut fusion_edit = None;
        let mut last_slot = -1;
        let team = m.team_load(world)?.clone();
        let fusions = team.fusions_load(world);
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
            if let Some(rep) = Representation::get(fusion.entity(), world) {
                rep.paint(r.rect, context, ui).log();
            }
            if r.clicked() {
                fusion_edit = Some(fusion.entity());
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
            if let Some(i) = r.dnd_release_payload::<i32>() {
                if slot != *i {
                    cn().reducers.match_reorder(*i as u8, slot as u8).unwrap();
                }
            }
        }

        if let Some(entity) = fusion_edit {
            Self::edit_fusion(entity, world)?;
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
                    cn().reducers.match_buy(sc.id()).unwrap();
                }
                let entity = world.get_id_link(sc.unit).unwrap();
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
    fn edit_fusion(entity: Entity, world: &mut World) -> Result<(), ExpressionError> {
        FusionEditorPlugin::edit_entity(entity, world, |f, world| {
            let m = Self::load_match(world)?;
            let fusions = m
                .team_load(world)?
                .fusions_load(world)
                .into_iter()
                .map(|f| f.to_strings_root())
                .collect_vec();
            for f in m
                .team_load(world)?
                .fusions_load(world)
                .iter()
                .map(|f| f.entity())
                .collect_vec()
            {
                world.entity_mut(f).despawn_recursive();
            }
            cn().reducers.match_edit_fusions(fusions).unwrap();
            GameState::Match.set_next(world);
            Ok(())
        })
    }
}
