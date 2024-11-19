use super::*;

pub struct IncubatorPlugin;

#[derive(Clone, Copy, EnumIter, Display)]
enum IncubatorTable {
    Unit,
    Trigger,
    Representation,
    House,
    Ability,
    Effect,
    Status,
}

#[derive(Resource, Default)]
struct NewUnit {
    name: String,
    description: String,
    pwr: i32,
    hp: i32,
}
#[derive(Resource, Default)]
struct NewTrigger {
    description: String,
    data: Trigger,
}
#[derive(Resource, Default)]
struct NewRepresentation {
    description: String,
    data: Representation,
}

impl IncubatorTable {
    fn id(self) -> String {
        self.to_string()
    }
    fn get_score(target: u64) -> i32 {
        cn().db
            .incubator_vote()
            .iter()
            .filter_map(|v| {
                if v.target == target {
                    Some(if v.vote { 1 } else { -1 })
                } else {
                    None
                }
            })
            .sum::<i32>()
    }
    fn get_link_id(from: u64, to: u64) -> Option<u64> {
        cn().db
            .incubator_link()
            .iter()
            .find(|l| l.from == from && l.to == to)
            .map(|l| l.id)
    }
    fn get_top_representation(from: u64) -> Option<String> {
        cn().db
            .incubator_link()
            .iter()
            .filter(|l| {
                l.from == from
                    && cn()
                        .db
                        .incubator_representation()
                        .id()
                        .find(&l.to)
                        .is_some()
            })
            .map(|l| (l.to, Self::get_score(l.id)))
            .max_by_key(|(_, score)| *score)
            .and_then(|(id, _)| {
                cn().db
                    .incubator_representation()
                    .id()
                    .find(&id)
                    .map(|d| d.data)
            })
    }
    fn open_create_popup(self, world: &mut World) {
        match self {
            IncubatorTable::Unit => {
                world.init_resource::<NewUnit>();
                Confirmation::new("New Unit".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2))
                    .accept(|world| {
                        if let Some(NewUnit {
                            name,
                            description,
                            pwr,
                            hp,
                        }) = world.remove_resource::<NewUnit>()
                        {
                            cn().reducers
                                .incubator_post_unit(name, description, hp, pwr)
                                .unwrap();
                        } else {
                            "New unit data not found".notify_error(world);
                        }
                    })
                    .accept_name("Post")
                    .cancel(|_| {})
                    .content(|ui, world| {
                        world.resource_scope(|_, mut unit: Mut<NewUnit>| {
                            Input::new("name").ui_string(&mut unit.name, ui);
                            Input::new("description").ui_string(&mut unit.description, ui);
                            DragValue::new(&mut unit.pwr).prefix("pwr:").ui(ui);
                            DragValue::new(&mut unit.hp).prefix("hp:").ui(ui);
                        })
                    })
                    .push(world);
            }
            IncubatorTable::Trigger => {
                world.init_resource::<NewTrigger>();
                Confirmation::new("New Trigger".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2))
                    .accept(|world| {
                        if let Some(NewTrigger { description, data }) =
                            world.remove_resource::<NewTrigger>()
                        {
                            cn().reducers
                                .incubator_post_trigger(ron::to_string(&data).unwrap(), description)
                                .unwrap();
                        } else {
                            "New trigger data not found".notify_error(world);
                        }
                    })
                    .accept_name("Post")
                    .cancel(|_| {})
                    .content(|ui, world| {
                        world.resource_scope(|world, mut trigger: Mut<NewTrigger>| {
                            Input::new("description").ui_string(&mut trigger.description, ui);
                            trigger
                                .data
                                .show_node("trigger", &Context::empty(), world, ui);
                        });
                    })
                    .push(world);
            }
            IncubatorTable::Representation => {
                world.init_resource::<NewRepresentation>();
                Confirmation::new(
                    "New Representation".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2),
                )
                .fullscreen()
                .accept(|world| {
                    if let Some(NewRepresentation { description, data }) =
                        world.remove_resource::<NewRepresentation>()
                    {
                        cn().reducers
                            .incubator_post_representation(
                                ron::to_string(&data).unwrap(),
                                description,
                            )
                            .unwrap();
                        world.insert_resource(NewRepresentation::default());
                    } else {
                        "New representation data not found".notify_error(world);
                    }
                })
                .accept_name("Post")
                .cancel(|_| {})
                .content(|ui, world| {
                    world.resource_scope(|world, mut representation: Mut<NewRepresentation>| {
                        Input::new("description").ui_string(&mut representation.description, ui);
                        representation.data.show_node(
                            "representation",
                            &Context::empty(),
                            world,
                            ui,
                        );
                    });
                })
                .push(world);
            }
            IncubatorTable::House => todo!(),
            IncubatorTable::Ability => todo!(),
            IncubatorTable::Effect => todo!(),
            IncubatorTable::Status => todo!(),
        }
    }
    fn open_tile(self, world: &mut World) {
        Tile::new(Side::Left, move |ui, world| {
            ui.horizontal(|ui| {
                if "Post New".cstr().button(ui).clicked() {
                    Self::open_create_popup(self, world);
                }
            });
            match self {
                IncubatorTable::Unit => {
                    let data = cn()
                        .db
                        .incubator_unit()
                        .iter()
                        .map(|d| {
                            let unit = TBaseUnit {
                                name: d.name.clone(),
                                pwr: d.pwr,
                                hp: d.hp,
                                rarity: 0,
                                house: "Default".into(),
                                pool: UnitPool::default(),
                                triggers: Vec::new(),
                                targets: Vec::new(),
                                effects: Vec::new(),
                                representation: Self::get_top_representation(d.id).unwrap_or_else(
                                    || ron::to_string(&Representation::default()).unwrap(),
                                ),
                            };
                            (d, unit)
                        })
                        .collect_vec();
                    Table::new("Units")
                        .row_height(64.0)
                        .column_base_unit_texture(|(_, u): &(TIncubatorUnit, TBaseUnit)| u)
                        .column_player_click("owner", |(d, _)| d.owner)
                        .column_cstr("name", |(d, _), _| d.name.clone())
                        .column_cstr("description", |(d, _), _| d.description.clone())
                        .column_int("pwr", |(d, _)| d.pwr)
                        .column_int("hp", |(d, _)| d.hp)
                        .column_btn("rep link", |d, ui, world| {
                            let from = d.0.id;
                            Confirmation::new(
                                "Representation Links".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2),
                            )
                            .accept(|_| {})
                            .accept_name("Close")
                            .content(move |ui, world| {
                                let data = cn()
                                    .db
                                    .incubator_representation()
                                    .iter()
                                    .map(|d| {
                                        (
                                            Self::get_score(
                                                Self::get_link_id(from, d.id).unwrap_or_default(),
                                            ),
                                            d,
                                        )
                                    })
                                    .collect_vec();
                                Table::new("Representation Links")
                                    .column_representation_texture(
                                        |d: &(i32, TIncubatorRepresentation)| {
                                            ron::from_str(&d.1.data).unwrap()
                                        },
                                    )
                                    .column_int("score", |(c, _)| *c)
                                    .column_btn_dyn(
                                        "+",
                                        Box::new(move |(_, d), _, _| {
                                            cn().reducers.incubator_link_add(from, d.id).unwrap();
                                        }),
                                    )
                                    .ui(&data, ui, world);
                            })
                            .push(world);
                        })
                        .ui(&data, ui, world);
                }
                IncubatorTable::Trigger => {
                    let data = cn().db.incubator_trigger().iter().collect_vec();
                    Table::new("Triggers")
                        .column_player_click("owner", |d: &TIncubatorTrigger| d.owner)
                        .column_cstr("description", |d, _| d.description.clone())
                        .column_cstr("data", |d, _| match ron::from_str::<Trigger>(&d.data) {
                            Ok(v) => v.cstr_expanded(),
                            Err(e) => format!("error: {e:?}").cstr_c(RED),
                        })
                        .ui(&data, ui, world);
                }
                IncubatorTable::Representation => {
                    let data = cn().db.incubator_representation().iter().collect_vec();
                    data.show_modified_table("Representations", ui, world, |t| {
                        t.column_btn_dyn(
                            "clone",
                            Box::new(move |d, _, world| {
                                world.insert_resource(NewRepresentation {
                                    description: d.description.clone(),
                                    data: ron::from_str(&d.data).unwrap(),
                                });
                                Self::open_create_popup(self, world);
                            }),
                        )
                    });
                }
                IncubatorTable::House => todo!(),
                IncubatorTable::Ability => todo!(),
                IncubatorTable::Effect => todo!(),
                IncubatorTable::Status => todo!(),
            }
        })
        .with_id(self.id())
        .transparent()
        .push(world);
    }
}

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                for table in IncubatorTable::iter() {
                    if table
                        .to_string()
                        .as_button()
                        .active(TilePlugin::is_open(&table.id(), world))
                        .ui(ui)
                        .clicked()
                    {
                        table.open_tile(world);
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
