use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            let data = TIncubator::iter().collect_vec();
            Table::new("Incubator")
                .title()
                .column_base_unit("unit", |d: &TIncubator| d.unit.last().unwrap().clone())
                .column_int("score", |d| {
                    TIncubatorVote::filter_by_target(d.id)
                        .map(|d| if d.vote { 1 } else { -1 })
                        .sum::<i32>()
                })
                .column_int("fav", |d| {
                    TIncubatorFavorite::filter_by_target(d.id).count() as i32
                })
                .column_user_click("owner", |d| d.owner)
                .column_btn_mod(
                    "+",
                    |d, _, _| {
                        incubator_vote(d.id, true);
                    },
                    |d, _, b| {
                        let v = TIncubatorVote::filter_by_owner(player_id())
                            .find(|v| v.target == d.id)
                            .map(|v| v.vote)
                            .unwrap_or_default();
                        b.active(v)
                    },
                )
                .column_btn_mod(
                    "-",
                    |d, _, _| {
                        incubator_vote(d.id, false);
                    },
                    |d, ui, b| {
                        let v = TIncubatorVote::filter_by_owner(player_id())
                            .find(|v| v.target == d.id)
                            .map(|v| !v.vote)
                            .unwrap_or_default();
                        b.red(ui).active(v)
                    },
                )
                .column_btn_mod(
                    "❤",
                    |d, _, _| {
                        incubator_favorite(d.id);
                    },
                    |d, _, b| {
                        let v = TIncubatorFavorite::find_by_owner(player_id())
                            .map(|v| v.target == d.id)
                            .unwrap_or_default();
                        b.active(v)
                    },
                )
                .column_cstr_click(
                    "open",
                    |_, _| "open".cstr_c(VISIBLE_LIGHT),
                    |d, world| {
                        let cards = d
                            .unit
                            .iter()
                            .map(|u| UnitCard::from_base(u.clone(), world).unwrap())
                            .collect_vec();
                        let own = d.owner == player_id();
                        let i = d.clone();
                        fn tile_id(id: u64) -> String {
                            format!("Incubator {}", id)
                        }
                        Tile::new(Side::Left, move |ui, world| {
                            cards.last().unwrap().ui(ui);
                            if cards.len() > 1 {
                                ui.collapsing("Previous versions", |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        "^".cstr().label(ui);
                                        for i in (0..cards.len() - 1).rev() {
                                            let name = cards[i].name.get_text();
                                            if Button::click(&name).ui(ui).clicked() {
                                                let card = cards[i].clone();
                                                Tile::new(Side::Left, move |ui, _| {
                                                    card.ui(ui);
                                                })
                                                .with_id(format!("{name}{i}"))
                                                .min_space(egui::vec2(300.0, 0.0))
                                                .push(world);
                                            }
                                            "<".cstr().label(ui);
                                        }
                                    });
                                });
                            }
                            if own {
                                ui.horizontal(|ui| {
                                    if Button::click("edit").ui(ui).clicked() {
                                        EditorPlugin::load_from_incubator(
                                            i.id,
                                            i.unit.last().unwrap().clone(),
                                            world,
                                        );
                                        GameState::Editor.proceed_to_target(world);
                                        return;
                                    }
                                    if Button::click("delete").red(ui).ui(ui).clicked() {
                                        let id = i.id;
                                        Confirmation::new("Delete Incubator unit".cstr_c(RED))
                                            .accept(move |_| {
                                                incubator_delete(id);
                                                on_incubator_delete(|_, _, status, id| {
                                                    let id = *id;
                                                    status.on_success(move |world| {
                                                        TilePlugin::close(&tile_id(id), world);
                                                        Notification::new_string(format!(
                                                            "Incubator entry#{id} deleted"
                                                        ))
                                                        .push(world)
                                                    });
                                                });
                                            })
                                            .cancel(|_| {})
                                            .push(world);
                                    }
                                });
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
                        .with_id(tile_id(d.id))
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
