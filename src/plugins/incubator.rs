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
    pub fn get_vote(owner: u64, target: &String) -> i32 {
        cn().db
            .incubator_vote()
            .id()
            .find(&format!("{owner}_{target}"))
            .map(|v| v.vote)
            .unwrap_or_default()
    }
}

impl IncubatorType {
    fn tile_id(&self) -> &str {
        self.as_ref()
    }
    fn add_tile(self, world: &mut World) {
        Tile::new(Side::Left, move |ui, world| {
            if "Add New".cstr().button(ui).clicked() {
                self.add_new(world);
            }
            match self {
                IncubatorType::UnitName => {
                    Table::new("Unit Names", |_| {
                        cn().db.incubator_unit_name().iter().collect_vec()
                    })
                    .column_cstr("name", |d: &TIncubatorUnitName, _| d.name.cstr())
                    .column_player_click("owner", |d| d.owner)
                    .column_btn_dyn(
                        "open",
                        Box::new(move |d, _, world| {
                            self.open(d.id, world);
                        }),
                    )
                    .ui(ui, world);
                }
                IncubatorType::UnitStats => {
                    Table::new("Unit Stats", |_| {
                        cn().db.incubator_unit_stats().iter().collect_vec()
                    })
                    .column_int("pwr", |d: &TIncubatorUnitStats| d.pwr)
                    .column_int("hp", |d| d.hp)
                    .column_player_click("owner", |d| d.owner)
                    .ui(ui, world);
                }
                IncubatorType::UnitRepresentation => {
                    Table::new("Unit Representations", |_| {
                        cn().db.incubator_unit_representation().iter().collect_vec()
                    })
                    .row_height(64.0)
                    .column_texture(Box::new(|d: &TIncubatorUnitRepresentation, world| {
                        TextureRenderPlugin::texture_representation_serialized(&d.data, world)
                    }))
                    .column_player_click("owner", |d| d.owner)
                    .ui(ui, world);
                }
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
            IncubatorType::UnitRepresentation => {
                #[derive(Resource, Default)]
                struct NewData {
                    rep: Representation,
                }
                world.init_resource::<NewData>();
                Confirmation::new("New Unit stats")
                    .content(move |ui, world| {
                        world.resource_scope(|world, mut r: Mut<NewData>| {
                            r.rep.show_node("", &Context::empty(), world, ui);
                        });
                    })
                    .cancel(|_| {})
                    .accept(|world| {
                        let data = world.remove_resource::<NewData>().unwrap();
                        cn().reducers
                            .incubator_post_unit_representation(ron::to_string(&data.rep).unwrap())
                            .unwrap();
                    })
                    .push(world);
            }
            IncubatorType::UnitTrigger => todo!(),
            IncubatorType::House => todo!(),
            IncubatorType::Ability => todo!(),
            IncubatorType::AbilityEffect => todo!(),
            IncubatorType::Status => todo!(),
            IncubatorType::StatusTrigger => todo!(),
        }
    }
    fn open_unit_stats_links(id: u64, world: &mut World) {
        #[derive(Resource)]
        struct Data {
            id: u64,
        }
        world.insert_resource(Data { id });
        Confirmation::new("Stats Links")
            .accept(|_| {})
            .accept_name("Close")
            .content(move |ui, world| {
                Table::new("Unit Stats", |world| {
                    let id = world.resource::<Data>().id;
                    cn().db
                        .incubator_unit_stats()
                        .iter()
                        .map(|d| (TIncubatorLink::find(id, d.id), d, id))
                        .sorted_by_key(|(l, _, _)| {
                            -l.as_ref().map(|l| l.score).unwrap_or(i32::MIN + 1)
                        })
                        .collect_vec()
                })
                .column_int(
                    "pwr",
                    |(_, d, _): &(Option<TIncubatorLink>, TIncubatorUnitStats, u64)| d.pwr,
                )
                .column_int("hp", |(_, d, _)| d.hp)
                .columns_incubator_vote_links(|(_, d, id)| format!("{id}_{}", d.id))
                .ui(ui, world);
            })
            .push(world);
    }
    fn open_unit_representation_links(id: u64, world: &mut World) {
        #[derive(Resource)]
        struct Data {
            id: u64,
        }
        world.insert_resource(Data { id });
        Confirmation::new("Representation Links")
            .accept(|_| {})
            .accept_name("Close")
            .content(move |ui, world| {
                Table::new("Unit Representation", |world| {
                    let id = world.resource::<Data>().id;
                    cn().db
                        .incubator_unit_representation()
                        .iter()
                        .map(|d| (TIncubatorLink::find(id, d.id), d, id))
                        .sorted_by_key(|(l, _, _)| {
                            -l.as_ref().map(|l| l.score).unwrap_or(i32::MIN + 1)
                        })
                        .collect_vec()
                })
                .column_texture(Box::new(
                    |(_, d, _): &(Option<TIncubatorLink>, TIncubatorUnitRepresentation, u64),
                     world| {
                        TextureRenderPlugin::texture_representation_serialized(&d.data, world)
                    },
                ))
                .columns_incubator_vote_links(|(_, d, id)| format!("{id}_{}", d.id))
                .ui(ui, world);
            })
            .push(world);
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
                                Self::open_unit_stats_links(id, world);
                            }
                            if "Representation Links".cstr().button(ui).clicked() {
                                Self::open_unit_representation_links(id, world);
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
