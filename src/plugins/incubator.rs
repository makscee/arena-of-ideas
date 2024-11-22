use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                for it in ContentType::iter() {
                    if it == ContentType::Data {
                        continue;
                    }
                    if it
                        .name()
                        .cstr_s(CstrStyle::Small)
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
    fn name(self) -> String {
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
        let table = Table::new("Content Table", |world| {
            let t: SContentType = world.resource::<TableContentType>().0.into();
            cn().db
                .content_piece()
                .iter()
                .filter(|p| p.t == t)
                .collect_vec()
        })
        .column_id("id", |d| d.id)
        .column_player_click("owner", |d| d.owner);
        let table = match self {
            ContentType::CUnit => table.column_cstr("name", |d, _| d.data.cstr()),
            ContentType::CUnitDescription => table.column_cstr("description", |d, _| d.data.cstr()),
            ContentType::CUnitStats => table.column_cstr("data", |d, _| d.data.cstr()),
            ContentType::CUnitTrigger => todo!(),
            ContentType::CUnitRepresentation => todo!(),
            ContentType::CAbility => todo!(),
            ContentType::CAbilityDescription => todo!(),
            ContentType::CHouse => todo!(),
            ContentType::CStatus => todo!(),
            ContentType::CStatusDescription => todo!(),
            ContentType::CStatusTrigger => todo!(),
            ContentType::CSummon => todo!(),
            ContentType::CAbilityEffect => todo!(),
            ContentType::Data => todo!(),
        }
        .column_btn("open", |d, _, world| {
            ContentType::from(d.t.clone()).open(d.id, world);
        });
        table.ui(ui, world);
    }
    fn add_tile(self, world: &mut World) {
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
        match self {
            ContentType::CUnit => {
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
                        cn().reducers
                            .incubator_post(SContentType::CUnit, name)
                            .unwrap();
                    })
                    .push(world);
            }
            ContentType::CUnitStats => {
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
                        let NewData { pwr, hp } = world.remove_resource::<NewData>().unwrap();
                        cn().reducers
                            .incubator_post(SContentType::CUnitStats, format!("{pwr}/{hp}"))
                            .unwrap();
                    })
                    .push(world);
            }
            ContentType::CUnitRepresentation => {
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
                            .incubator_post(
                                SContentType::CUnitRepresentation,
                                ron::to_string(&data.rep).unwrap(),
                            )
                            .unwrap();
                    })
                    .push(world);
            }
            ContentType::CUnitDescription => todo!(),
            ContentType::CUnitTrigger => todo!(),
            ContentType::CAbility => todo!(),
            ContentType::CAbilityDescription => todo!(),
            ContentType::CHouse => todo!(),
            ContentType::CStatus => todo!(),
            ContentType::CStatusDescription => todo!(),
            ContentType::CStatusTrigger => todo!(),
            ContentType::CSummon => todo!(),
            ContentType::CAbilityEffect => todo!(),
            ContentType::Data => todo!(),
        }
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
                self.show(&piece.data, ui);
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
    fn show(self, data: &str, ui: &mut Ui) {
        for link in self.content_piece().content_links() {
            if link == ContentType::Data {
                continue;
            }
            link.name().button(ui);
        }
        match self {
            ContentType::CUnit => {
                format!("name: {}", data.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold)).label(ui);
            }
            ContentType::CUnitDescription => {
                format!("description: {}", data.cstr()).label(ui);
            }
            ContentType::CUnitStats => match self.parse_stats(data) {
                Ok((pwr, hp)) => {
                    format!("pwr: {}", pwr.cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)).label(ui);
                    format!("hp: {}", hp.cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)).label(ui);
                }
                Err(e) => {
                    format!(
                        "[red Stats parse error:] {}",
                        e.cstr_cs(RED, CstrStyle::Bold)
                    )
                    .label(ui);
                }
            },
            ContentType::CUnitTrigger => todo!(),
            ContentType::CUnitRepresentation => todo!(),
            ContentType::CAbility => todo!(),
            ContentType::CAbilityDescription => todo!(),
            ContentType::CHouse => todo!(),
            ContentType::CStatus => todo!(),
            ContentType::CStatusDescription => todo!(),
            ContentType::CStatusTrigger => todo!(),
            ContentType::CSummon => todo!(),
            ContentType::CAbilityEffect => todo!(),
            ContentType::Data => todo!(),
        }
    }

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

    fn content_piece(self) -> Box<dyn ContentPiece> {
        match self {
            ContentType::Data => Box::new(String::new()),
            ContentType::CUnit => Box::new(CUnit::default()),
            ContentType::CUnitDescription => Box::new(CUnitDescription::default()),
            ContentType::CUnitStats => Box::new(CUnitStats::default()),
            ContentType::CUnitTrigger => Box::new(CUnitTrigger::default()),
            ContentType::CUnitRepresentation => Box::new(CUnitRepresentation::default()),
            ContentType::CAbility => Box::new(CAbility::default()),
            ContentType::CAbilityEffect => Box::new(CAbilityEffect::default()),
            ContentType::CAbilityDescription => Box::new(CAbilityDescription::default()),
            ContentType::CHouse => Box::new(CHouse::default()),
            ContentType::CStatus => Box::new(CStatus::default()),
            ContentType::CStatusDescription => Box::new(CStatusDescription::default()),
            ContentType::CStatusTrigger => Box::new(CStatusTrigger::default()),
            ContentType::CSummon => Box::new(CSummon::default()),
        }
    }
}

impl TContentLink {
    fn find(from: u64, to: u64) -> Option<Self> {
        cn().db
            .content_link()
            .from_to()
            .find(&format!("{from}_{to}"))
    }
}

trait ContentPiece {
    fn content_type(&self) -> ContentType;
    fn content_links(&self) -> Vec<ContentType>;
}

impl ContentPiece for String {
    fn content_type(&self) -> ContentType {
        ContentType::Data
    }
    fn content_links(&self) -> Vec<ContentType> {
        default()
    }
}
impl ContentPiece for CUnit {
    fn content_type(&self) -> ContentType {
        ContentType::CUnit
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CUnit {
            name,
            description,
            stats,
            representation,
        } = self;
        [
            name.content_type(),
            description.content_type(),
            stats.content_type(),
            representation.content_type(),
        ]
        .into()
    }
}
impl ContentPiece for CUnitDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitDescription
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CUnitDescription { text, trigger } = self;
        [text.content_type(), trigger.content_type()].into()
    }
}
impl ContentPiece for CUnitStats {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitStats
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CUnitStats { data } = self;
        [data.content_type()].into()
    }
}
impl ContentPiece for CUnitRepresentation {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitRepresentation
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CUnitRepresentation { data } = self;
        [data.content_type()].into()
    }
}
impl ContentPiece for CUnitTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitTrigger
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CUnitTrigger { data, ability } = self;
        [data.content_type(), ability.content_type()].into()
    }
}
impl ContentPiece for CAbility {
    fn content_type(&self) -> ContentType {
        ContentType::CAbility
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CAbility {
            name,
            description,
            house,
        } = self;
        [
            name.content_type(),
            description.content_type(),
            house.content_type(),
        ]
        .into()
    }
}
impl ContentPiece for CAbilityDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CAbilityDescription
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CAbilityDescription { text, effect } = self;
        [text.content_type(), effect.content_type()].into()
    }
}
impl ContentPiece for CAbilityEffect {
    fn content_type(&self) -> ContentType {
        ContentType::CAbilityEffect
    }
    fn content_links(&self) -> Vec<ContentType> {
        match self {
            CAbilityEffect::Status(status) => [status.content_type()].into(),
            CAbilityEffect::Summon(summon) => [summon.content_type()].into(),
            CAbilityEffect::Action(s) => [s.content_type()].into(),
        }
    }
}
impl ContentPiece for CStatus {
    fn content_type(&self) -> ContentType {
        ContentType::CStatus
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CStatus { name, description } = self;
        [name.content_type(), description.content_type()].into()
    }
}
impl ContentPiece for CSummon {
    fn content_type(&self) -> ContentType {
        ContentType::CSummon
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CSummon { name, stats } = self;
        [name.content_type(), stats.content_type()].into()
    }
}
impl ContentPiece for CStatusDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusDescription
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CStatusDescription { data, trigger } = self;
        [data.content_type(), trigger.content_type()].into()
    }
}
impl ContentPiece for CStatusTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusTrigger
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CStatusTrigger { data } = self;
        [data.content_type()].into()
    }
}
impl ContentPiece for CHouse {
    fn content_type(&self) -> ContentType {
        ContentType::CHouse
    }
    fn content_links(&self) -> Vec<ContentType> {
        let CHouse { data } = self;
        [data.content_type()].into()
    }
}

impl Default for CUnit {
    fn default() -> Self {
        Self {
            name: default(),
            description: default(),
            stats: default(),
            representation: default(),
        }
    }
}
impl Default for CUnitDescription {
    fn default() -> Self {
        Self {
            text: default(),
            trigger: default(),
        }
    }
}
impl Default for CUnitStats {
    fn default() -> Self {
        Self { data: default() }
    }
}
impl Default for CUnitRepresentation {
    fn default() -> Self {
        Self { data: default() }
    }
}
impl Default for CUnitTrigger {
    fn default() -> Self {
        Self {
            data: default(),
            ability: default(),
        }
    }
}
impl Default for CAbility {
    fn default() -> Self {
        Self {
            name: default(),
            description: default(),
            house: default(),
        }
    }
}
impl Default for CAbilityDescription {
    fn default() -> Self {
        Self {
            text: default(),
            effect: default(),
        }
    }
}
impl Default for CAbilityEffect {
    fn default() -> Self {
        Self::Action(default())
    }
}
impl Default for CStatusDescription {
    fn default() -> Self {
        Self {
            data: default(),
            trigger: default(),
        }
    }
}
impl Default for CStatus {
    fn default() -> Self {
        Self {
            name: default(),
            description: default(),
        }
    }
}
impl Default for CStatusTrigger {
    fn default() -> Self {
        Self { data: default() }
    }
}
impl Default for CSummon {
    fn default() -> Self {
        Self {
            name: default(),
            stats: default(),
        }
    }
}
impl Default for CHouse {
    fn default() -> Self {
        Self { data: default() }
    }
}
