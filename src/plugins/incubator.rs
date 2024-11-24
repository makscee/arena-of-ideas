use std::collections::VecDeque;

use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            // CUnit::default().show_node(ui, world);
            // CUnit {
            //     name: "Unit_Name".into(),
            //     description: CUnitDescription {
            //         text: "Unit description bla bla".into(),
            //         trigger: CUnitTrigger {
            //             data: "Unit trigger data".into(),
            //             ability: CAbility {
            //                 name: "Ability_Name".into(),
            //                 description: CAbilityDescription {
            //                     text: "Ability description bla bla".into(),
            //                     effect: CAbilityEffect::Status(CStatus {
            //                         name: "Some_Status_Name".into(),
            //                         description: CStatusDescription {
            //                             text: "Status Description bla bla".into(),
            //                             trigger: CStatusTrigger {
            //                                 data: "Status trigger data".into(),
            //                             },
            //                         },
            //                     }),
            //                 },
            //                 house: CHouse {
            //                     data: "HouseName/#ff00ff".into(),
            //                 },
            //             },
            //         },
            //     },
            //     stats: CUnitStats {
            //         data: "13/9".into(),
            //     },
            //     representation: CUnitRepresentation {
            //         data: r#"(material:Shape(shape:Circle(radius:Sum(Sum(F(0.81),Mul(Index,F(-0.17))),Mul(Beat,F(0.05)))),shape_type:Line(thickness:F(2.27)),fill:GradientLinear(point1:V2(0.0,-0.5),point2:V2(0.0,0.5),parts:[F(0.0),Sum(F(0.99),Mul(Index,F(0.0)))],colors:[OwnerState(Color),HexColor("00000000")]),fbm:None,alpha:F(1.0),padding:F(0.0)),children:[],mapping:{Offset:Vec2EE(Zero,Sum(Mul(Index,F(-0.04)),Mul(Sub(Zero,Abs(Beat)),Mul(F(0.1),Index))))},count:3)"#.into(),
            //     },
            // }
            // .show_node(ui, world);
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
        let table = table
            .column(
                "data",
                |_, _| default(),
                |d, _, ui, world| {
                    d.t.to_local().show(&d.data, ui, world);
                },
                false,
            )
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
                    color: Color32,
                }
                world.init_resource::<NewData>();
                add_new_popup(
                    move |ui, world| {
                        let mut r = world.resource_mut::<NewData>();
                        Input::new("name").ui_string(&mut r.name, ui);
                        ui.color_edit_button_srgba(&mut r.color);
                    },
                    move |world| {
                        let NewData { name, color } = world.remove_resource::<NewData>().unwrap();
                        Ok((self, format!("{name}/{}", color.to_hex())))
                    },
                    world,
                );
            }
            ContentType::CAction => todo!(),
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
        Tile::new(Side::Left, move |ui, world| {
            let mut p = self.content_piece();
            p.visit(id, |parent, t, data| {
                *data = t.find_data().unwrap_or_default();
            });
            p.show_node(ui, world);
        })
        .transparent()
        .push(world);
        return;
        Confirmation::new("Content Piece")
            .content(move |ui, world| {
                let Some(piece) = cn().db.content_piece().id().find(&id) else {
                    return;
                };
                if piece.owner == player_id() && "Delete".cstr_c(RED).button(ui).clicked() {
                    Self::delete(self, id, world);
                }
                self.show(&piece.data, ui, world);
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
    fn show(self, data: &str, ui: &mut Ui, world: &mut World) {
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
            ContentType::CAction => {
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
            ContentType::CAction => Box::new(CAction::default()),
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
    fn to(&self) -> &str {
        self.from_to.split_once('_').unwrap().1
    }
    fn find(from: u64, to: u64) -> Option<Self> {
        cn().db
            .content_link()
            .from_to()
            .find(&format!("{from}_{to}"))
    }
    fn collect(from: u64, to_type: ContentType) -> Vec<TContentLink> {
        let prefix = format!("{from}_");
        let t: SContentType = to_type.into();
        cn().db
            .content_link()
            .iter()
            .filter(|l| {
                l.from_to.starts_with(&prefix)
                    && u64::from_str(l.to()).is_ok_and(|id| {
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

trait ContentPiece {
    fn content_type(&self) -> ContentType;
    fn inner(&self) -> Vec<Box<dyn ContentPiece>>;
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String));
    fn data(&self) -> &str;
    fn inject_data(&mut self, data: String);
    fn show_node(&self, ui: &mut Ui, world: &mut World) {
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(4.0),
            outer_margin: Margin::ZERO,
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: STROKE_DARK,
        };
        FRAME.show(ui, |ui| {
            let ct = self.content_type();
            ui.horizontal(|ui| {
                if ct
                    .name()
                    .cstr_cs(CYAN, CstrStyle::Bold)
                    .button(ui)
                    .clicked()
                {
                    ct.add_tile(world);
                }
                ct.show(self.data(), ui, world);
            });

            ui.vertical(|ui| {
                for i in self.inner() {
                    i.show_node(ui, world);
                }
            })
        });
    }
}

fn id_by_data(data: &String) -> u64 {
    cn().db
        .content_piece()
        .data()
        .find(data)
        .map(|d| d.id)
        .unwrap_or_default()
}

impl ContentPiece for CUnit {
    fn content_type(&self) -> ContentType {
        ContentType::CUnit
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnit {
            data: _,
            description,
            stats,
            representation,
        } = self;
        vec![
            Box::new(description.clone()),
            Box::new(stats.clone()),
            Box::new(representation.clone()),
        ]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, _: u64, f: fn(u64, ContentType, &mut String)) {
        let parent = id_by_data(&self.data);
        self.description.visit(parent, f);
        self.stats.visit(parent, f);
    }
}
impl ContentPiece for CUnitDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitDescription
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnitDescription { data: _, trigger } = self;
        vec![Box::new(trigger.clone())]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.trigger.visit(parent, f);
    }
}
impl ContentPiece for CUnitStats {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitStats
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        default()
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CUnitRepresentation {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitRepresentation
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnitRepresentation { data: _ } = self;
        default()
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CUnitTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CUnitTrigger
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CUnitTrigger { data: _, ability } = self;
        vec![Box::new(ability.clone())]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.ability.visit(parent, f);
    }
}
impl ContentPiece for CAbility {
    fn content_type(&self) -> ContentType {
        ContentType::CAbility
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CAbility {
            data: _,
            description,
            house,
        } = self;
        vec![Box::new(description.clone()), Box::new(house.clone())]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.description.visit(parent, f);
        self.house.visit(parent, f);
    }
}
impl ContentPiece for CAbilityDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CAbilityDescription
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CAbilityDescription {
            data: _,
            status,
            summon,
            action,
        } = self;
        vec![
            Box::new(status.clone().unwrap_or_default()),
            Box::new(summon.clone().unwrap_or_default()),
            Box::new(action.clone().unwrap_or_default()),
        ]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        if let Some(c) = &mut self.status {
            c.visit(parent, f);
        }
        if let Some(c) = &mut self.summon {
            c.visit(parent, f);
        }
        if let Some(c) = &mut self.action {
            c.visit(parent, f);
        }
    }
}
impl ContentPiece for CAction {
    fn content_type(&self) -> ContentType {
        ContentType::CAction
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CAction { data: _ } = self;
        default()
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CStatus {
    fn content_type(&self) -> ContentType {
        ContentType::CStatus
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CStatus {
            data: _,
            description,
        } = self;
        vec![Box::new(description.clone())]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.description.visit(parent, f);
    }
}
impl ContentPiece for CSummon {
    fn content_type(&self) -> ContentType {
        ContentType::CSummon
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CSummon { data: _, stats } = self;
        vec![Box::new(stats.clone())]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.stats.visit(parent, f);
    }
}
impl ContentPiece for CStatusDescription {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusDescription
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CStatusDescription { data: _, trigger } = self;
        vec![Box::new(trigger.clone())]
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
        let parent = id_by_data(&self.data);
        self.trigger.visit(parent, f);
    }
}
impl ContentPiece for CStatusTrigger {
    fn content_type(&self) -> ContentType {
        ContentType::CStatusTrigger
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CStatusTrigger { data: _ } = self;
        default()
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}
impl ContentPiece for CHouse {
    fn content_type(&self) -> ContentType {
        ContentType::CHouse
    }
    fn inner(&self) -> Vec<Box<dyn ContentPiece>> {
        let CHouse { data: _ } = self;
        default()
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn inject_data(&mut self, data: String) {
        self.data = data;
    }
    fn visit(&mut self, parent: u64, f: fn(u64, ContentType, &mut String)) {
        f(parent, self.content_type(), &mut self.data);
    }
}

impl Default for CUnit {
    fn default() -> Self {
        Self {
            data: default(),
            description: default(),
            stats: default(),
            representation: default(),
        }
    }
}
impl Default for CUnitDescription {
    fn default() -> Self {
        Self {
            data: default(),
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
            data: default(),
            description: default(),
            house: default(),
        }
    }
}
impl Default for CAbilityDescription {
    fn default() -> Self {
        Self {
            data: default(),
            status: Some(default()),
            summon: Some(default()),
            action: Some(default()),
        }
    }
}
impl Default for CAction {
    fn default() -> Self {
        Self { data: default() }
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
            data: default(),
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
            data: default(),
            stats: default(),
        }
    }
}
impl Default for CHouse {
    fn default() -> Self {
        Self { data: default() }
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
}
