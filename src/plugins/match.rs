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
    fn shop_slot_rect(i: usize, max: usize, ui: &mut Ui) -> Rect {
        let rect = ui.available_rect_before_wrap();
        let mut rect = rect.with_max_y(rect.max.y / 2.0);
        let w = rect.width() / max as f32;
        rect.set_width(w.at_most(rect.height()));
        rect.translate(egui::vec2(w * i as f32, 0.0))
    }
    pub fn open_shop_window(world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        Window::new("Match", move |ui, world| {
            let md = world.resource::<MatchData>();
            md.g.cstr().label(ui);
            let shop_units = &md.shop_units;
            for (i, su) in shop_units.iter().enumerate() {
                let Some(unit) = su else {
                    continue;
                };
                let rep = unit.representation.as_ref().unwrap();
                let context = Context::default().set_owner_node(unit).take();
                let rect = Self::shop_slot_rect(i, shop_units.len(), ui);
                ui.painter().rect_stroke(rect, ROUNDING, STROKE_DARK);
                let rect = rect.shrink(15.0);
                RepresentationPlugin::paint_rect(rect, &context, &unit_rep().material, ui).log();
                RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui).log();
            }
        })
        .push(world);
    }
}
