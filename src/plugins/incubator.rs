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
    fn open_create_popup(self, world: &mut World) {
        match self {
            IncubatorTable::Unit => {
                world.insert_resource(NewUnit::default());
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
                world.insert_resource(NewTrigger::default());
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
                world.insert_resource(NewRepresentation::default());
                Confirmation::new(
                    "New Representation".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Heading2),
                )
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
                    let data = cn().db.incubator_unit().iter().collect_vec();
                    Table::new("Units")
                        .column_player_click("owner", |d: &TIncubatorUnit| d.owner)
                        .column_cstr("name", |d, _| d.name.clone())
                        .column_cstr("description", |d, _| d.description.clone())
                        .column_int("pwr", |d| d.pwr)
                        .column_int("hp", |d| d.hp)
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
                    Table::new("Representations")
                        .column_player_click("owner", |d: &TIncubatorRepresentation| d.owner)
                        .column_cstr("description", |d, _| d.description.clone())
                        .column_cstr("data", |d, _| match ron::from_str::<Trigger>(&d.data) {
                            Ok(v) => v.cstr_expanded(),
                            Err(e) => format!("error: {e:?}").cstr_c(RED),
                        })
                        .ui(&data, ui, world);
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
