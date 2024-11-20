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
                        .column_btn_mod_dyn(
                            "delete",
                            Box::new(move |d, _, world| {
                                let id = d.id;
                                Confirmation::new("Delete unit name?")
                                    .accept(move |_| {
                                        cn().reducers.incubator_delete(id, self.into()).unwrap()
                                    })
                                    .cancel(|_| {})
                                    .push(world);
                            }),
                            Box::new(|d: &TIncubatorUnitName, _, b: Button| {
                                b.enabled(d.owner == player_id())
                            }),
                        )
                        .ui(&data, ui, world);
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
        })
        .with_id(self.tile_id().into())
        .transparent()
        .push(world);
    }
    fn add_new(self, world: &mut World) {
        match self {
            IncubatorType::UnitName => {
                #[derive(Resource, Default)]
                struct NewUnitName {
                    name: String,
                }
                world.init_resource::<NewUnitName>();
                Confirmation::new("New Unit name")
                    .content(move |ui, world| {
                        Input::new("name")
                            .ui_string(&mut world.resource_mut::<NewUnitName>().name, ui);
                    })
                    .cancel(|_| {})
                    .accept(|world| {
                        let name = world.remove_resource::<NewUnitName>().unwrap().name;
                        cn().reducers.incubator_post_unit_name(name).unwrap();
                    })
                    .push(world);
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
}
