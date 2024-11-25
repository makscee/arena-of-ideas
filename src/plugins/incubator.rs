use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            ui.vertical(|ui| {
                for p in ContentType::iter() {
                    if p.name()
                        .as_button()
                        .active(TilePlugin::is_open(p.tile_id(), world))
                        .ui(ui)
                        .clicked()
                    {
                        p.add_tile(world);
                    }
                }
                space(ui);
                if "post test data".to_owned().button(ui).clicked() {
                    CUnit {
                        data: "TestUnit".into(),
                        description: CUnitDescription {
                            data: "Test unit description".into(),
                            trigger: CUnitTrigger {
                                data: ron::to_string(&Trigger::Fire {
                                    triggers: vec![(FireTrigger::BattleStart, None)],
                                    targets: default(),
                                    effects: vec![(Effect::Kill, None)],
                                })
                                .unwrap(),
                                ability: CAbility {
                                    data: "AbilityName".into(),
                                    description: CAbilityDescription {
                                        data: "Ability description".into(),
                                        status: Some(CStatus {
                                            data: "Status_Name".into(),
                                            description: CStatusDescription {
                                                data: "Status description".into(),
                                                trigger: CStatusTrigger {
                                                    data: ron::to_string(&Trigger::Fire {
                                                        triggers: vec![(
                                                            FireTrigger::TurnEnd,
                                                            None,
                                                        )],
                                                        targets: default(),
                                                        effects: vec![(Effect::Kill, None)],
                                                    })
                                                    .unwrap(),
                                                },
                                            },
                                        }),
                                        summon: Some(CSummon {
                                            data: "SummonName".into(),
                                            stats: CUnitStats { data: "2/1".into() },
                                        }),
                                        action: Some(CEffect {
                                            data: ron::to_string(&Effect::Damage).unwrap(),
                                        }),
                                    },
                                    house: CHouse {
                                        data: "TestHouse/#ff00ff".into(),
                                    },
                                },
                            },
                        },
                        stats: CUnitStats { data: "1/1".into() },
                        representation: CUnitRepresentation {
                            data: ron::to_string(
                                &game_assets()
                                    .heroes
                                    .values()
                                    .choose(&mut thread_rng())
                                    .unwrap()
                                    .trigger,
                            )
                            .unwrap(),
                        },
                    }
                    .visit(0, |_, t, data| {
                        cn().reducers
                            .incubator_post(t.to_server(), data.clone())
                            .unwrap();
                    });
                }
            });
        })
        .keep()
        .transparent()
        .pinned()
        .no_frame()
        .push(world);
    }
    pub fn get_vote(owner: u64, target: &String) -> i8 {
        cn().db
            .content_vote()
            .id()
            .find(&format!("{owner}_{target}"))
            .map(|v| v.vote)
            .unwrap_or_default()
    }
}

impl ContentType {
    pub fn name(self) -> String {
        self.as_ref().trim_start_matches('C').to_case(Case::Title)
    }
    fn tile_id(&self) -> &str {
        self.as_ref()
    }
    fn show_table(self, ui: &mut Ui, world: &mut World) {
        title(&self.name(), ui);
        #[derive(Resource)]
        struct TableContentType(ContentType);
        world.insert_resource(TableContentType(self));
        Table::new("Content Table", |world| {
            let t: SContentType = world.resource::<TableContentType>().0.into();
            cn().db
                .content_piece()
                .iter()
                .filter(|p| p.t == t)
                .collect_vec()
        })
        .add_content_piece_columns(|d| d.clone())
        .add_content_vote_columns(|d| d.id.to_string())
        .add_content_favorite_columns(|d| (d.t.to_local().to_string(), d.id.to_string()))
        .column_btn("open", |d, _, world| {
            ContentType::from(d.t.clone()).open(d.id, world);
        })
        .ui(ui, world);
    }
    pub fn open_links(self, id: u64, world: &mut World) {
        Confirmation::new("Links")
            .accept(|_| {})
            .accept_name("Close")
            .content(move |ui, world| {
                Self::show_links(self, id, ui, world);
            })
            .push(world);
    }
    fn show_links(self, id: u64, ui: &mut Ui, world: &mut World) {
        let cp = cn().db.content_piece().id().find(&id).unwrap();
        ui.vertical_centered_justified(|ui| {
            format!("Links from [b {}] to [b {self}]", cp.t.to_local()).label(ui);
        });
        cp.t.to_local().show(&cp.data, ui, world);
        #[derive(Resource, Clone)]
        struct LinkData {
            id: u64,
            t: ContentType,
        }
        world.insert_resource(LinkData { id, t: self });
        Table::new("Active Links", |world| {
            let LinkData { id, t } = world.resource::<LinkData>().clone();
            let type_key = format!("{id}_{t}");
            TContentVoteScore::collect_links(id, t)
                .into_iter()
                .map(|d| (d, type_key.clone()))
                .collect_vec()
        })
        .add_content_piece_columns(|(s, _)| {
            cn().db
                .content_piece()
                .id()
                .find(&s.link_to_u64().unwrap())
                .unwrap()
        })
        .add_content_vote_columns(|(d, _)| d.id.clone())
        .add_content_favorite_columns(|(d, type_key)| (type_key.clone(), d.id.clone()))
        .title()
        .ui(ui, world);

        Table::new("New Links", |world| {
            let LinkData { id, t } = world.resource::<LinkData>().clone();
            let links: HashSet<u64> = HashSet::from_iter(
                TContentVoteScore::collect_links(id, t)
                    .into_iter()
                    .filter_map(|l| l.link_to_u64()),
            );
            let t = t.to_server();
            cn().db
                .content_piece()
                .iter()
                .filter(|p| p.t == t && !links.contains(&p.id))
                .map(|p| (format!("{id}_{}", p.id), p))
                .collect_vec()
        })
        .add_content_piece_columns(|d| d.1.clone())
        .add_content_vote_columns(|d| d.0.clone())
        .title()
        .ui(ui, world);
    }
    pub fn add_tile(self, world: &mut World) {
        Tile::new(Side::Left, move |ui, world| {
            if "Add New".cstr().button(ui).clicked() {
                self.add_new(world);
            }
            self.show_table(ui, world);
        })
        .with_id(self.tile_id().into())
        .transparent()
        .push(world);
    }
    fn add_new(self, world: &mut World) {
        fn add_new_popup(
            content: impl Fn(&mut Ui, &mut World) + Send + Sync + 'static,
            post: impl FnOnce(&mut World) -> Result<(ContentType, String), String>
                + Send
                + Sync
                + 'static,
            world: &mut World,
        ) {
            Confirmation::new("Add New")
                .content(content)
                .cancel(|_| {})
                .accept(|world| {
                    match post(world) {
                        Ok((t, data)) => cn().reducers.incubator_post(t.into(), data).unwrap(),
                        Err(e) => e.notify_error(world),
                    };
                })
                .push(world);
        }
        match self {
            ContentType::CUnit
            | ContentType::CAbility
            | ContentType::CStatus
            | ContentType::CSummon => {
                #[derive(Resource, Default)]
                struct NewData {
                    name: String,
                }
                world.init_resource::<NewData>();
                add_new_popup(
                    move |ui, world| {
                        Input::new("name").ui_string(&mut world.resource_mut::<NewData>().name, ui);
                    },
                    move |world| {
                        let name = world.remove_resource::<NewData>().unwrap().name;
                        Ok((self, name))
                    },
                    world,
                );
            }
            ContentType::CUnitDescription
            | ContentType::CAbilityDescription
            | ContentType::CStatusDescription => {
                #[derive(Resource, Default)]
                struct NewData {
                    data: String,
                }
                world.init_resource::<NewData>();
                add_new_popup(
                    move |ui, world| {
                        Input::new("description")
                            .ui_string(&mut world.resource_mut::<NewData>().data, ui);
                    },
                    move |world| {
                        let data = world.remove_resource::<NewData>().unwrap().data;
                        Ok((self, data))
                    },
                    world,
                );
            }
            ContentType::CUnitStats => {
                #[derive(Resource, Default)]
                struct NewData {
                    pwr: i32,
                    hp: i32,
                }
                world.init_resource::<NewData>();
                add_new_popup(
                    move |ui, world| {
                        let mut r = world.resource_mut::<NewData>();
                        DragValue::new(&mut r.pwr).prefix("pwr:").ui(ui);
                        DragValue::new(&mut r.hp).prefix("hp:").ui(ui);
                    },
                    move |world| {
                        let NewData { pwr, hp } = world.remove_resource::<NewData>().unwrap();
                        Ok((self, format!("{pwr}/{hp}")))
                    },
                    world,
                );
            }
            ContentType::CUnitRepresentation => {
                #[derive(Resource, Default)]
                struct NewData {
                    data: Representation,
                }
                world.init_resource::<NewData>();
                add_new_popup(
                    move |ui, world| {
                        world.resource_scope(|world, mut r: Mut<NewData>| {
                            r.data.show_node("", &Context::empty(), world, ui);
                        });
                    },
                    move |world| {
                        let data = world.remove_resource::<NewData>().unwrap();
                        ron::to_string(&data.data)
                            .map(|v| (self, v))
                            .map_err(|e| e.to_string())
                    },
                    world,
                );
            }
            ContentType::CUnitTrigger | ContentType::CStatusTrigger => {
                #[derive(Resource, Default)]
                struct NewData {
                    data: Trigger,
                }
                world.init_resource::<NewData>();

                add_new_popup(
                    move |ui, world| {
                        world.resource_scope(|world, mut r: Mut<NewData>| {
                            r.data.show_node("", &Context::empty(), world, ui);
                        });
                    },
                    move |world| {
                        let data = world.remove_resource::<NewData>().unwrap();
                        ron::to_string(&data.data)
                            .map(|v| (self, v))
                            .map_err(|e| e.to_string())
                    },
                    world,
                );
            }
            ContentType::CHouse => {
                #[derive(Resource, Default)]
                struct NewData {
                    name: String,
                    color: [u8; 3],
                }
                world.init_resource::<NewData>();
                add_new_popup(
                    move |ui, world| {
                        let mut r = world.resource_mut::<NewData>();
                        Input::new("name").ui_string(&mut r.name, ui);
                        ui.color_edit_button_srgb(&mut r.color);
                    },
                    move |world| {
                        let NewData { name, color: c } =
                            world.remove_resource::<NewData>().unwrap();
                        let color = Color32::from_rgb(c[0], c[1], c[2]);
                        Ok((self, format!("{name}/{}", color.to_hex())))
                    },
                    world,
                );
            }
            ContentType::CEffect => {
                #[derive(Resource, Default)]
                struct NewData {
                    data: Effect,
                }
                world.init_resource::<NewData>();
                add_new_popup(
                    move |ui, world| {
                        world.resource_scope(|world, mut r: Mut<NewData>| {
                            r.data.show_node("", &Context::empty(), world, ui);
                        });
                    },
                    move |world| {
                        let data = world.remove_resource::<NewData>().unwrap();
                        ron::to_string(&data.data)
                            .map(|v| (self, v))
                            .map_err(|e| e.to_string())
                    },
                    world,
                );
            }
        }
    }
    fn find_data(self) -> Option<String> {
        let t: SContentType = self.into();
        cn().db
            .content_piece()
            .iter()
            .filter(|p| p.t == t)
            .min_by_key(|p| p.id)
            .map(|p| p.data)
    }
    fn open(self, id: u64, world: &mut World) {
        Confirmation::new("Content Piece")
            .content(move |ui, world| {
                let Some(piece) = cn().db.content_piece().id().find(&id) else {
                    return;
                };
                if piece.owner == player_id() && "Delete".cstr_c(RED).button(ui).clicked() {
                    Self::delete(self, id, world);
                }
                let mut p = self.content_piece();
                *p.data_mut() = cn().db.content_piece().id().find(&id).unwrap().data;
                p.visit(id, |parent, t, data| {
                    *data = t.find_data().unwrap_or_default();
                });
                p.show_node(id, ui, world);
            })
            .accept(|_| {})
            .accept_name("Close")
            .push(world);
    }
    fn delete(self, id: u64, world: &mut World) {
        Confirmation::new("Delete content piece?")
            .content(move |ui, _| {
                format!("Content piece #{id} will be deleted")
                    .cstr_c(RED)
                    .label(ui);
            })
            .accept(move |world| {
                cn().reducers.incubator_delete(id).unwrap();
                Confirmation::pop(world);
            })
            .cancel(|_| {})
            .push(world);
    }
    fn error_cstr(e: &str) -> Cstr {
        e.cstr_cs(RED, CstrStyle::Small)
    }
    fn show_error(e: &str, ui: &mut Ui) {
        Self::error_cstr(e).as_label(ui).truncate().ui(ui);
    }
    pub fn show(self, data: &str, ui: &mut Ui, world: &mut World) {
        if data.is_empty() {
            "empty".cstr_cs(VISIBLE_DARK, CstrStyle::Small).label(ui);
            return;
        }
        match self {
            ContentType::CUnit
            | ContentType::CAbility
            | ContentType::CStatus
            | ContentType::CSummon => {
                ("name: ".to_owned() + &data.cstr_cs(name_color(data), CstrStyle::Bold)).label(ui);
            }
            ContentType::CUnitDescription
            | ContentType::CStatusDescription
            | ContentType::CAbilityDescription => {
                format!("description: {}", data.cstr()).label(ui);
            }
            ContentType::CUnitStats => match self.parse_stats(data) {
                Ok((pwr, hp)) => {
                    format!("pwr: [b [yellow {pwr}]] hp: [b [red {hp}]]").label(ui);
                }
                Err(e) => Self::show_error(&e, ui),
            },
            ContentType::CUnitTrigger | ContentType::CStatusTrigger => {
                match self.parse_trigger(data) {
                    Ok(v) => {
                        v.cstr_expanded().label(ui);
                    }
                    Err(e) => Self::show_error(&e, ui),
                }
            }
            ContentType::CUnitRepresentation => match self.parse_representation(data) {
                Ok(v) => {
                    let tex = TextureRenderPlugin::texture_representation(&v, world);
                    show_texture(128.0, tex, ui);
                    v.cstr_expanded().label(ui);
                }
                Err(e) => Self::show_error(&e, ui),
            },
            ContentType::CHouse => match self.parse_house(data) {
                Ok((name, color)) => {
                    name.cstr_cs(color, CstrStyle::Bold).label(ui);
                }
                Err(e) => Self::show_error(&e, ui),
            },
            ContentType::CEffect => {
                data.cstr().label(ui);
            }
        }
    }

    fn content_piece(self) -> Box<dyn ContentPiece> {
        match self {
            ContentType::CUnit => Box::new(CUnit::default()),
            ContentType::CUnitDescription => Box::new(CUnitDescription::default()),
            ContentType::CUnitStats => Box::new(CUnitStats::default()),
            ContentType::CUnitTrigger => Box::new(CUnitTrigger::default()),
            ContentType::CUnitRepresentation => Box::new(CUnitRepresentation::default()),
            ContentType::CAbility => Box::new(CAbility::default()),
            ContentType::CEffect => Box::new(CEffect::default()),
            ContentType::CAbilityDescription => Box::new(CAbilityDescription::default()),
            ContentType::CHouse => Box::new(CHouse::default()),
            ContentType::CStatus => Box::new(CStatus::default()),
            ContentType::CStatusDescription => Box::new(CStatusDescription::default()),
            ContentType::CStatusTrigger => Box::new(CStatusTrigger::default()),
            ContentType::CSummon => Box::new(CSummon::default()),
        }
    }
}

impl TContentVoteScore {
    fn link_to(&self) -> &str {
        self.id.split_once('_').unwrap().1
    }
    fn link_to_u64(&self) -> Option<u64> {
        u64::from_str(self.link_to()).ok()
    }
    fn find_link(from: u64, to: u64) -> Option<Self> {
        cn().db
            .content_vote_score()
            .id()
            .find(&format!("{from}_{to}"))
    }
    fn collect_links(from: u64, to_type: ContentType) -> Vec<TContentVoteScore> {
        let prefix = format!("{from}_");
        let t: SContentType = to_type.into();
        cn().db
            .content_vote_score()
            .iter()
            .filter(|l| {
                l.id.starts_with(&prefix)
                    && u64::from_str(l.link_to()).is_ok_and(|id| {
                        cn().db
                            .content_piece()
                            .id()
                            .find(&id)
                            .is_some_and(|p| p.t == t)
                    })
            })
            .collect_vec()
    }
}

impl ContentType {
    fn parse_stats(self, data: &str) -> Result<(i32, i32), String> {
        match self {
            ContentType::CUnitStats => {
                let Some((pwr, hp)) = data.split_once('/') else {
                    return Err("Failed to parse stats".into());
                };
                let pwr = i32::from_str(pwr).map_err(|e| e.to_string())?;
                let hp = i32::from_str(hp).map_err(|e| e.to_string())?;
                Ok((pwr, hp))
            }
            _ => Err(format!(
                "Wrong content type. Expected: {} Got {self}",
                ContentType::CUnitStats
            )),
        }
    }
    fn parse_trigger(self, data: &str) -> Result<Trigger, String> {
        match self {
            ContentType::CUnitTrigger | ContentType::CStatusTrigger => {
                match ron::from_str::<Trigger>(data) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(format!("Failed to parse {self}: {e}")),
                }
            }
            _ => Err(format!(
                "Wrong content type. Expected: {} or {} Got {self}",
                ContentType::CUnitTrigger,
                ContentType::CStatusTrigger,
            )),
        }
    }
    fn parse_representation(self, data: &str) -> Result<Representation, String> {
        match self {
            ContentType::CUnitRepresentation => match ron::from_str::<Representation>(data) {
                Ok(v) => Ok(v),
                Err(e) => Err(format!("Failed to parse {self}: {e}")),
            },
            _ => Err(format!(
                "Wrong content type. Expected: {} Got {self}",
                ContentType::CUnitRepresentation
            )),
        }
    }
    fn parse_house(self, data: &str) -> Result<(String, Color32), String> {
        match self {
            ContentType::CHouse => match data.split_once('/') {
                Some((name, color)) => {
                    let color = Color32::from_hex(color)
                        .map_err(|e| format!("Failed to parse color: {e:?}"))?;
                    Ok((name.to_owned(), color))
                }
                None => Err(format!("Failed to parse {self}")),
            },
            _ => Err(format!(
                "Wrong content type. Expected: {} Got {self}",
                ContentType::CHouse
            )),
        }
    }
    fn parse_effect(self, data: &str) -> Result<Effect, String> {
        match self {
            ContentType::CEffect => match ron::from_str::<Effect>(data) {
                Ok(v) => Ok(v),
                Err(e) => Err(format!("Failed to parse {self}: {e}")),
            },
            _ => Err(format!(
                "Wrong content type. Expected: {} Got {self}",
                ContentType::CEffect
            )),
        }
    }
}
