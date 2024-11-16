use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn tile_id(id: u64) -> String {
        format!("Incubator {}", id)
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            let data = cn().db.incubator().iter().collect_vec();
            Table::new("Incubator")
                .title()
                .column_base_unit("unit", |d: &TIncubator| d.unit.last().unwrap().clone())
                .column_int("score", |d| {
                    cn().db
                        .incubator_vote()
                        .iter()
                        .filter(|v| v.target == d.id)
                        .map(|d| if d.vote { 1 } else { -1 })
                        .sum::<i32>()
                })
                .column_int("fav", |d| {
                    cn().db
                        .incubator_favorite()
                        .iter()
                        .filter(|f| f.target == d.id)
                        .count() as i32
                })
                .column_user_click("owner", |d| d.owner)
                .column_btn_mod(
                    "+",
                    |d, _, _| {
                        let _ = cn().reducers.incubator_vote_set(d.id, true);
                    },
                    |d, _, b| {
                        let v = cn()
                            .db
                            .incubator_vote()
                            .iter()
                            .find(|v| v.owner == player_id() && v.target == d.id)
                            .map(|v| v.vote)
                            .unwrap_or_default();
                        b.active(v)
                    },
                )
                .column_btn_mod(
                    "-",
                    |d, _, _| {
                        let _ = cn().reducers.incubator_vote_set(d.id, false);
                    },
                    |d, ui, b| {
                        let v = cn()
                            .db
                            .incubator_vote()
                            .iter()
                            .find(|v| v.owner == player_id() && v.target == d.id)
                            .map(|v| !v.vote)
                            .unwrap_or_default();
                        b.red(ui).active(v)
                    },
                )
                .column_btn_mod(
                    "❤",
                    |d, _, _| {
                        let _ = cn().reducers.incubator_favorite_set(d.id);
                    },
                    |d, _, b| {
                        let v = cn()
                            .db
                            .incubator_favorite()
                            .iter()
                            .find(|f| f.owner == player_id())
                            .map(|f| f.target == d.id)
                            .unwrap_or_default();
                        b.enabled(d.owner != player_id()).active(v)
                    },
                )
                .column_cstr_click(
                    "open",
                    |_, _| "open".cstr_c(VISIBLE_LIGHT),
                    |d, world| {
                        let unit = d.unit.last().unwrap().clone();
                        let cards = d
                            .unit
                            .iter()
                            .map(|u| UnitCard::from_base(u.clone(), world).unwrap())
                            .collect_vec();
                        let own = d.owner == player_id();
                        let i = d.clone();
                        Tile::new(Side::Left, move |ui, world| {
                            cards.last().unwrap().ui(ui);
                            if cards.len() > 1 {
                                ui.collapsing("Previous versions", |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        "^".cstr().label(ui);
                                        for i in (0..cards.len() - 1).rev() {
                                            let name = cards[i].name.get_text();
                                            if Button::new(&name).ui(ui).clicked() {
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
                                    if Button::new("edit").ui(ui).clicked() {
                                        EditorPlugin::load_from_incubator(
                                            i.id,
                                            i.unit.last().unwrap().clone(),
                                            world,
                                        );
                                        GameState::Editor.proceed_to_target(world);
                                        return;
                                    }
                                    if Button::new("delete").red(ui).ui(ui).clicked() {
                                        let id = i.id;
                                        Confirmation::new("Delete Incubator unit".cstr_c(RED))
                                            .accept(move |_| {
                                                let _ = cn().reducers.incubator_delete(id);
                                            })
                                            .cancel(|_| {})
                                            .push(world);
                                    }
                                });
                            } else {
                                if Button::new("open in editor").ui(ui).clicked() {
                                    EditorPlugin::load_unit(
                                        i.unit.last().unwrap().clone().into(),
                                        world,
                                    );
                                    GameState::Editor.proceed_to_target(world);
                                    return;
                                }
                            }
                            if Button::new("spawn").ui(ui).clicked() {
                                world.game_clear();
                                let unit = PackedUnit::from(unit.clone()).unpack(
                                    TeamPlugin::entity(Faction::Team, world),
                                    None,
                                    None,
                                    world,
                                );
                                UnitPlugin::place_into_slot(unit, world);
                            }
                        })
                        .with_id(Self::tile_id(d.id))
                        .min_space(egui::vec2(300.0, 0.0))
                        .push(world);
                    },
                )
                .ui(&data, ui, world);
        })
        .pinned()
        .push(world);
        Tile::new(Side::Bottom, |ui, world| {
            TeamContainer::new(Faction::Team).slots(1).ui(ui, world);
        })
        .transparent()
        .pinned()
        .push(world);
    }
}
