use super::*;

pub struct TextColumnPlugin;

#[derive(Component, Default)]
pub struct TextColumn {
    lines: Vec<(f32, Cstr)>,
}

impl Plugin for TextColumnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui);
    }
}

impl TextColumnPlugin {
    pub fn add(entity: Entity, text: Cstr, world: &mut World) {
        if let Some(mut tc) = world.get_mut::<TextColumn>(entity) {
            tc.lines.push((gt().insert_head(), text));
        }
    }
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };
        let mut drawn: Vec<Vec<Rect>> = [default()].into();
        let mut prev_lvl: HashMap<Entity, usize> = default();
        let t = gt().play_head();
        let start_height = world_to_screen(vec3(0.0, 2.0, 0.0), world).y;
        const Y_PER_LEVEL: f32 = 22.0;
        const LIFETIME: f32 = 4.0;
        const EASE_IN: f32 = 0.3;
        const EASE_OUT: f32 = 0.5;
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                let mut lines = world
                    .query::<(&TextColumn, &Transform, Entity)>()
                    .iter(world)
                    .map(|(tc, tr, e)| {
                        (
                            tc.lines
                                .iter()
                                .filter(|(ts, _)| *ts < t && *ts + LIFETIME > t)
                                .rev(),
                            world_to_screen(tr.translation, world).x,
                            e,
                        )
                    })
                    .sorted_by(|(_, x1, _), (_, x2, _)| x1.total_cmp(&x2))
                    .collect_vec();
                while !lines.is_empty() {
                    let mut remove: Vec<usize> = default();
                    for (i, (line, x, entity)) in lines.iter_mut().enumerate() {
                        if let Some((ts, text)) = line.next() {
                            let ts = t - *ts;
                            let a = smoothstep(0.0, EASE_IN, ts)
                                .min(1.0 - smoothstep(LIFETIME - EASE_OUT, LIFETIME, ts))
                                .clamp(0.0, 1.0);
                            let (_, galley, _) = text.as_label_alpha(a, ui).layout_in_ui(ui);
                            let rect = galley.rect;
                            let rect = rect.translate(egui::vec2(*x - rect.width() * 0.5, 0.0));
                            let mut cur_lvl = prev_lvl.get(entity).copied().unwrap_or_default();
                            while drawn[cur_lvl].iter().any(|r| r.intersects(rect)) {
                                cur_lvl += 1;
                                if drawn.get(cur_lvl).is_none() {
                                    drawn.push(default());
                                }
                            }
                            drawn[cur_lvl].push(rect);
                            prev_lvl.insert(*entity, cur_lvl);
                            let rect = rect.translate(egui::vec2(
                                0.0,
                                start_height - cur_lvl as f32 * Y_PER_LEVEL,
                            ));
                            ui.painter().add(epaint::TextShape::new(
                                rect.left_top(),
                                galley,
                                LIGHT_GRAY,
                            ));
                        } else {
                            remove.push(i);
                            continue;
                        }
                    }
                    for i in remove.into_iter().rev() {
                        let _ = lines.remove(i);
                    }
                }
            });
    }
}
