use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Resource)]
struct MatchData {
    g: u32,
    shop_units: Vec<Option<Unit>>,
}

impl MatchPlugin {
    pub fn load_match_data(id: u64, world: &mut World) {
        let m = Match::from_table(NodeDomain::Match, id).unwrap();
        world.insert_resource(MatchData {
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
            g: m.g,
        });
    }
    fn shop_slot_rect(i: usize, max: usize, mut rect: Rect) -> Rect {
        rect.set_height(rect.height() * 0.5);
        let w = rect.width() / max as f32;
        rect.set_width(w.at_most(rect.height()));
        rect.translate(egui::vec2(w * i as f32, 0.0))
    }
    fn show_shop_slot(rect: Rect, slot: usize, ui: &mut Ui) -> (Rect, Response) {
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
        let ui = &mut ui.child_ui(rect, Layout::bottom_up(Align::Center), None);
        ui.with_layout(
            Layout::bottom_up(Align::Center).with_cross_justify(true),
            |ui| {
                if ui.button("Buy").clicked() {
                    cn().reducers.match_buy(slot as u8).unwrap();
                }
            },
        );
        let rect = ui.available_rect_before_wrap();
        let size = rect.width().at_most(rect.height());
        let rect = Rect::from_center_size(rect.center(), egui::vec2(size, size));
        let slot_resp = ui.allocate_rect(rect, Sense::click());
        if slot_resp.hovered() {
            ui.painter().rect_stroke(rect, ROUNDING, STROKE_YELLOW);
        } else {
            ui.painter().rect_stroke(rect, ROUNDING, STROKE_DARK);
        }
        (rect, slot_resp)
    }
    pub fn open_shop_window(world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
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
        Window::new("Match", move |ui, world| {
            let md = world.remove_resource::<MatchData>().unwrap();
            md.g.cstr().label(ui);
            let shop_units = &md.shop_units;
            let height = ui.available_rect_before_wrap().height();
            ui.columns(shop_units.len(), |ui| {
                for (i, su) in shop_units.iter().enumerate() {
                    let Some(unit) = su else {
                        continue;
                    };
                    let ui = &mut ui[i];
                    let rect = ui.available_rect_before_wrap();
                    let rect = rect.with_max_y(rect.min.y + rect.height() * 0.5);
                    let (rect, resp) = Self::show_shop_slot(rect, i, ui);
                    let context = Context::default().set_owner_node(unit).take();
                    if resp.hovered() {
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
            });
            world.insert_resource(md);
        })
        .default_width(500.0)
        .push(world);
    }
}
