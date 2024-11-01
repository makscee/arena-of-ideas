use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            let data = TIncubator::iter().collect_vec();
            Table::new("Incubator")
                .title()
                .column_base_unit("unit", |d: &TIncubator| d.unit.last().unwrap().clone())
                .column_user_click("owner", |d| d.owner)
                .column_cstr_click(
                    "open",
                    |_, _| "open".cstr_c(VISIBLE_LIGHT),
                    |d, world| {
                        let cards = d
                            .unit
                            .iter()
                            .map(|u| UnitCard::from_base(u.clone(), world).unwrap())
                            .collect_vec();
                        let own = d.owner == user_id();
                        let i = d.clone();
                        Tile::new(Side::Left, move |ui, world| {
                            cards.last().unwrap().ui(ui);
                            if cards.len() > 1 {
                                ui.collapsing("Previous versions", |ui| {
                                    ui.horizontal(|ui| {
                                        "^".cstr().label(ui);
                                        for i in (0..cards.len() - 1).rev() {
                                            let name = cards[i].name.get_text();
                                            if Button::click(&name).ui(ui).clicked() {
                                                let card = cards[i].clone();
                                                Tile::new(Side::Left, move |ui, _| {
                                                    card.ui(ui);
                                                })
                                                .with_id(format!("{name}{i}"))
                                                .push(world);
                                            }
                                            "<".cstr().label(ui);
                                        }
                                    });
                                });
                            }
                            if own {
                                if Button::click("edit").ui(ui).clicked() {
                                    EditorPlugin::load_from_incubator(
                                        i.id,
                                        i.unit.last().unwrap().clone(),
                                        world,
                                    );
                                    GameState::Editor.proceed_to_target(world);
                                    return;
                                }
                            } else {
                                if Button::click("open in editor").ui(ui).clicked() {
                                    EditorPlugin::load_unit(
                                        i.unit.last().unwrap().clone().into(),
                                        world,
                                    );
                                    GameState::Editor.proceed_to_target(world);
                                    return;
                                }
                            }
                        })
                        .with_id(format!("Incubator {}", d.id))
                        .min_space(egui::vec2(300.0, 0.0))
                        .push(world);
                    },
                )
                .ui(&data, ui, world);
        })
        .pinned()
        .push(world);
    }
}
