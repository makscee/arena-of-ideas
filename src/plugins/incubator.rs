use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                for it in IncubatorType::iter() {
                    if it
                        .to_string()
                        .to_case(Case::Title)
                        .as_button()
                        .active(TilePlugin::is_open(it.tile_id(), world))
                        .ui(ui)
                        .clicked()
                    {
                        it.add_tile(world);
                    }
                }
            });
        })
        .no_expand()
        .keep()
        .transparent()
        .pinned()
        .push(world);
    }
}

impl IncubatorType {
    fn tile_id(&self) -> &str {
        self.as_ref()
    }
    fn get_vote(owner: u64, target: String) -> i32 {
        cn().db
            .incubator_vote()
            .id()
            .find(&format!("{owner}_{target}"))
            .map(|v| v.vote)
            .unwrap_or_default()
    }
    fn add_tile(self, world: &mut World) {
        Tile::new(Side::Left, move |ui, world| {
            if "Add New".cstr().button(ui).clicked() {
                self.add_new(world);
            }
            match self {
                IncubatorType::UnitName => {
                    let data = cn().db.incubator_unit_name().iter().collect_vec();
                    Table::new("Unit Names")
                        .column_cstr("name", |d: &TIncubatorUnitName, _| d.name.cstr())
                        .column_player_click("owner", |d| d.owner)
                        .column_btn_dyn(
                            "open",
                            Box::new(move |d, _, world| {
                                self.open(d.id, world);
                            }),
                        )
                        .ui(&data, ui, world);
                }
                IncubatorType::UnitStats => {
                    let data = cn().db.incubator_unit_stats().iter().collect_vec();
                    Table::new("Unit Stats")
                        .column_int("pwr", |d: &TIncubatorUnitStats| d.pwr)
                        .column_int("hp", |d| d.hp)
                        .column_player_click("owner", |d| d.owner)
                        .ui(&data, ui, world);
                }
                IncubatorType::UnitRepresentation => todo!(),
                IncubatorType::UnitTrigger => todo!(),
                IncubatorType::House => todo!(),
                IncubatorType::Ability => todo!(),
                IncubatorType::AbilityEffect => todo!(),
                IncubatorType::Status => todo!(),
                IncubatorType::StatusTrigger => todo!(),
            }
        })
        .with_id(self.tile_id().into())
        .transparent()
        .push(world);
    }
    fn add_new(self, world: &mut World) {
        match self {
            IncubatorType::UnitName => {
                #[derive(Resource, Default)]
                struct NewData {
                    name: String,
                }
                world.init_resource::<NewData>();
                Confirmation::new("New Unit name")
                    .content(move |ui, world| {
                        Input::new("name").ui_string(&mut world.resource_mut::<NewData>().name, ui);
                    })
                    .cancel(|_| {})
                    .accept(|world| {
                        let name = world.remove_resource::<NewData>().unwrap().name;
                        cn().reducers.incubator_post_unit_name(name).unwrap();
                    })
                    .push(world);
            }
            IncubatorType::UnitStats => {
                #[derive(Resource, Default)]
                struct NewData {
                    pwr: i32,
                    hp: i32,
                }
                world.init_resource::<NewData>();
                Confirmation::new("New Unit stats")
                    .content(move |ui, world| {
                        let mut r = world.resource_mut::<NewData>();
                        DragValue::new(&mut r.pwr).prefix("pwr:").ui(ui);
                        DragValue::new(&mut r.hp).prefix("hp:").ui(ui);
                    })
                    .cancel(|_| {})
                    .accept(|world| {
                        let data = world.remove_resource::<NewData>().unwrap();
                        cn().reducers
                            .incubator_post_unit_stats(data.pwr, data.hp)
                            .unwrap();
                    })
                    .push(world);
            }
            IncubatorType::UnitRepresentation => todo!(),
            IncubatorType::UnitTrigger => todo!(),
            IncubatorType::House => todo!(),
            IncubatorType::Ability => todo!(),
            IncubatorType::AbilityEffect => todo!(),
            IncubatorType::Status => todo!(),
            IncubatorType::StatusTrigger => todo!(),
        }
    }
    fn open(self, id: u64, world: &mut World) {
        match self {
            IncubatorType::UnitName => {
                if let Some(data) = cn().db.incubator_unit_name().id().find(&id) {
                    Confirmation::new(&self.as_ref().to_case(Case::Title))
                        .content(move |ui, world| {
                            ui.vertical_centered_justified(|ui| {
                                data.name
                                    .cstr_cs(VISIBLE_LIGHT, CstrStyle::Heading2)
                                    .label(ui);
                            });
                            if "Stats Links".cstr().button(ui).clicked() {
                                Confirmation::new("Stats Links")
                                    .accept(|_| {})
                                    .accept_name("Close")
                                    .content(move |ui, world| {
                                        let data = cn()
                                            .db
                                            .incubator_unit_stats()
                                            .iter()
                                            .map(|d| (TIncubatorLink::find(id, d.id), d))
                                            .sorted_by_key(|(l, _)| {
                                                -l.as_ref().map(|l| l.score).unwrap_or(i32::MIN + 1)
                                            })
                                            .collect_vec();
                                        Table::new("Unit Stats")
                                            .column_int(
                                                "score",
                                                |d: &(
                                                    Option<TIncubatorLink>,
                                                    TIncubatorUnitStats,
                                                )| {
                                                    d.0.as_ref()
                                                        .map(|l| l.score)
                                                        .unwrap_or_default()
                                                },
                                            )
                                            .column_int("pwr", |(_, d)| d.pwr)
                                            .column_int("hp", |(_, d)| d.hp)
                                            .column_btn_mod_dyn(
                                                "-",
                                                Box::new(|(l, _), _, _| {
                                                    cn().reducers
                                                        .incubator_vote_set(
                                                            l.as_ref().unwrap().id.clone(),
                                                            -1,
                                                        )
                                                        .unwrap();
                                                }),
                                                Box::new(move |(l, _), ui, b| {
                                                    if let Some(l) = l {
                                                        b.active(
                                                            Self::get_vote(
                                                                player_id(),
                                                                l.id.clone(),
                                                            ) == -1,
                                                        )
                                                        .red(ui)
                                                    } else {
                                                        b.enabled(false)
                                                    }
                                                }),
                                            )
                                            .column_btn_mod_dyn(
                                                "+",
                                                Box::new(move |(l, d), _, _| {
                                                    if let Some(l) = l {
                                                        cn().reducers
                                                            .incubator_vote_set(l.id.clone(), -1)
                                                            .unwrap();
                                                    } else {
                                                        cn().reducers
                                                            .incubator_link_add(
                                                                id.to_string(),
                                                                SIncubatorType::UnitName,
                                                                d.id.to_string(),
                                                                SIncubatorType::UnitStats,
                                                            )
                                                            .unwrap();
                                                    }
                                                }),
                                                Box::new(move |(l, _), _, b| {
                                                    if let Some(l) = l {
                                                        b.active(
                                                            Self::get_vote(
                                                                player_id(),
                                                                l.id.clone(),
                                                            ) == 1,
                                                        )
                                                    } else {
                                                        b
                                                    }
                                                }),
                                            )
                                            .ui(&data, ui, world);
                                    })
                                    .push(world);
                            }
                            if data.owner == player_id() {
                                if "Delete".to_owned().as_button().red(ui).ui(ui).clicked() {
                                    self.delete(id, world);
                                }
                            }
                        })
                        .accept(|_| {})
                        .accept_name("Close")
                        .push(world);
                }
            }
            IncubatorType::UnitStats => todo!(),
            IncubatorType::UnitRepresentation => todo!(),
            IncubatorType::UnitTrigger => todo!(),
            IncubatorType::House => todo!(),
            IncubatorType::Ability => todo!(),
            IncubatorType::AbilityEffect => todo!(),
            IncubatorType::Status => todo!(),
            IncubatorType::StatusTrigger => todo!(),
        }
    }
    fn delete(self, id: u64, world: &mut World) {
        Confirmation::new("Delete unit name?")
            .accept(move |_| cn().reducers.incubator_delete(id, self.into()).unwrap())
            .cancel(|_| {})
            .push(world);
    }
}

impl TIncubatorLink {
    fn get_from(&self) -> u64 {
        self.id.split_once('_').unwrap().0.parse::<u64>().unwrap()
    }
    fn get_to(&self) -> u64 {
        self.id.split_once('_').unwrap().1.parse::<u64>().unwrap()
    }
    fn find(from: u64, to: u64) -> Option<Self> {
        cn().db.incubator_link().id().find(&format!("{from}_{to}"))
    }
}
