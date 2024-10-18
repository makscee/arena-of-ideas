use super::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorResource>()
            .add_systems(OnEnter(GameState::Editor), Self::on_enter)
            .add_systems(OnExit(GameState::Editor), Self::on_exit);
    }
}

#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub struct EditorResource {
    mode: Mode,

    team: PackedTeam,

    battle: (PackedTeam, PackedTeam),

    unit: PackedUnit,
    unit_entity: Option<Entity>,
    unit_mode: UnitMode,

    representation: Representation,
    vfx: Vfx,
}
fn rm(world: &mut World) -> Mut<EditorResource> {
    world.resource_mut::<EditorResource>()
}

#[derive(
    Default, EnumIter, AsRefStr, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Debug,
)]
enum Mode {
    #[default]
    Team,
    Battle,
    Unit,
    Representation,
    Vfx,
}

#[derive(
    Default, EnumIter, AsRefStr, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Debug,
)]
enum UnitMode {
    #[default]
    Unit,
    Trigger,
    Rep,
}

impl Into<String> for Mode {
    fn into(self) -> String {
        self.as_ref().to_case(Case::Title).to_string()
    }
}
impl Into<String> for UnitMode {
    fn into(self) -> String {
        self.as_ref().to_case(Case::Title).to_string()
    }
}

impl EditorPlugin {
    fn on_enter(world: &mut World) {
        *rm(world) = client_state().editor.clone();
    }
    fn on_exit(world: &mut World) {
        Self::clear(world);
    }
    fn clear(world: &mut World) {
        world.game_clear();
        TilePlugin::clear(world);
    }
    fn save_state(world: &mut World) {
        let mut cs = client_state().clone();
        cs.editor = rm(world).clone();
        cs.save();
    }
    fn refresh(world: &mut World) {
        world.game_clear();
        match rm(world).mode {
            Mode::Team => {
                rm(world).team.clone().unpack(Faction::Team, world);
                UnitPlugin::place_into_slots(world);
            }
            Mode::Battle => todo!(),
            Mode::Unit => {
                let entity = rm(world).unit.clone().unpack(
                    TeamPlugin::entity(Faction::Team, world),
                    Some(0),
                    None,
                    world,
                );
                UnitPlugin::place_into_slot(entity, world);
                rm(world).unit_entity = Some(entity);
            }
            Mode::Representation => todo!(),
            Mode::Vfx => todo!(),
        }
    }
    fn get_team_unit(slot: usize, world: &mut World) -> Option<PackedUnit> {
        rm(world).team.units.get(slot).cloned()
    }
    fn set_team_unit(slot: usize, unit: PackedUnit, world: &mut World) {
        let team = &mut rm(world).team;
        if team.units.len() > slot {
            team.units.remove(slot);
            team.units.insert(slot, unit);
        } else {
            team.units.push(unit);
        }
    }
    fn remove_team_unit(slot: usize, world: &mut World) {
        let team = &mut rm(world).team;
        if team.units.len() < slot {
            return;
        }
        team.units.remove(slot);
    }
    fn load_mode(world: &mut World) {
        let mode = rm(world).mode;
        info!("Load editor mode {mode}");
        Self::clear(world);
        Self::save_state(world);
        match mode {
            Mode::Team => {
                rm(world).team.clone().unpack(Faction::Team, world);

                Tile::new(Side::Top, |ui, world| {
                    TeamContainer::new(Faction::Team)
                        .top_content(|ui, _| {
                            if Button::click("Load own").ui(ui).clicked() {
                                Confirmation::new("Open own team".cstr())
                                    .content(|ui, world| {
                                        TTeam::filter_by_owner(user_id())
                                            .filter(|t| t.pool.eq(&TeamPool::Owned))
                                            .collect_vec()
                                            .show_modified_table("Teams", ui, world, |t| {
                                                t.column_btn("select", |t, ui, world| {
                                                    rm(world).team = PackedTeam::from_id(t.id);
                                                    Confirmation::close_current(
                                                        &egui_context(world).unwrap(),
                                                    );
                                                    Self::load_mode(world);
                                                })
                                            });
                                    })
                                    .cancel(|_| {})
                                    .push(ui.ctx());
                            }
                        })
                        .slot_content(|slot, e, ui, world| {
                            if e.is_none() {
                                return;
                            }
                            if Button::click("Edit").ui(ui).clicked() {
                                let mut rm = rm(world);
                                rm.unit = rm.team.units[slot].clone();
                                rm.mode = Mode::Unit;
                                Self::load_mode(world);
                            }
                        })
                        .context_menu(|slot, entity, ui, world| {
                            ui.reset_style();
                            ui.set_min_width(150.0);
                            if Button::click("Paste").ui(ui).clicked() {
                                if let Some(v) = paste_from_clipboard(world) {
                                    match ron::from_str::<PackedUnit>(&v) {
                                        Ok(v) => {
                                            Self::set_team_unit(slot, v, world);
                                            Self::refresh(world);
                                        }
                                        Err(e) => {
                                            format!("Failed to deserialize unit from {v}: {e}")
                                                .notify_error(world)
                                        }
                                    }
                                } else {
                                    "Clipboard is empty".notify_error(world);
                                }
                                ui.close_menu();
                            }
                            if entity.is_none() {
                                if Button::click("Spawn default").ui(ui).clicked() {
                                    Self::set_team_unit(slot, default(), world);
                                    Self::refresh(world);
                                    ui.close_menu();
                                }
                            } else {
                                if Button::click("Copy").ui(ui).clicked() {
                                    if let Some(unit) = Self::get_team_unit(slot, world) {
                                        match ron::to_string(&unit) {
                                            Ok(v) => copy_to_clipboard(&v, world),
                                            Err(e) => format!("Failed to serialize unit: {e}")
                                                .notify_error(world),
                                        }
                                    }
                                    ui.close_menu();
                                }
                                if Button::click("Delete").red(ui).ui(ui).clicked() {
                                    Self::remove_team_unit(slot, world);
                                    Self::refresh(world);
                                    ui.close_menu();
                                }
                            }
                        })
                        .ui(ui, world);
                })
                .stretch_min()
                .transparent()
                .pinned()
                .push(world);
            }
            Mode::Battle => {}
            Mode::Unit => {
                Self::refresh(world);
                Tile::new(Side::Right, |ui, world| {
                    TeamContainer::new(Faction::Team)
                        .slots(1)
                        .hug_unit()
                        .slot_content(|_, _, ui, world| {
                            if let Some(entity) = rm(world).unit_entity {
                                match UnitCard::new(&Context::new(entity), world) {
                                    Ok(c) => {
                                        ui.horizontal(|ui| {
                                            if Button::click("Copy").ui(ui).clicked() {
                                                copy_to_clipboard(
                                                    &ron::to_string(&rm(world).unit).unwrap(),
                                                    world,
                                                );
                                            }
                                            if Button::click("Paste").ui(ui).clicked() {
                                                let s = paste_from_clipboard(world);
                                                match s {
                                                    Some(s) => {
                                                        match ron::from_str::<PackedUnit>(&s) {
                                                            Ok(v) => {
                                                                rm(world).unit = v;
                                                                Self::refresh(world);
                                                            }
                                                            Err(e) => format!(
                                                                "Unit deserialize error: {e}"
                                                            )
                                                            .notify_error(world),
                                                        }
                                                    }
                                                    None => {
                                                        "Clipboard is empty".notify_error(world)
                                                    }
                                                }
                                            }
                                        });
                                        ScrollArea::vertical().show(ui, |ui| c.ui(ui));
                                    }
                                    Err(e) => error!("UnitCard error: {e}"),
                                }
                            }
                        })
                        .ui(ui, world);
                })
                .stretch_min()
                .transparent()
                .pinned()
                .push(world);
                Tile::new(Side::Top, |ui, world| {
                    EnumSwitcher::show(&mut rm(world).unit_mode, ui);
                })
                .pinned()
                .no_expand()
                .transparent()
                .push(world);
                Tile::new(Side::Left, |ui, world| {
                    ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| match rm(world).unit_mode {
                            UnitMode::Unit => {
                                let mut unit = rm(world).unit.clone();
                                Self::unit_edit(&mut unit, ui);
                                if rm(world).unit != unit {
                                    rm(world).unit = unit;
                                    Self::refresh(world);
                                }
                            }
                            UnitMode::Trigger => {
                                let r = rm(world);
                                let mut trigger = r.unit.trigger.clone();
                                let context = &Context::new(r.unit_entity.unwrap());
                                Self::trigger_edit(&mut trigger, context, ui, world);
                                if rm(world).unit.trigger != trigger {
                                    rm(world).unit.trigger = trigger;
                                    Self::refresh(world);
                                }
                            }
                            UnitMode::Rep => {
                                let r = rm(world);
                                let context = Context::new(r.unit_entity.unwrap())
                                    .set_var(VarName::Index, 1.into())
                                    .take();
                                let mut rep = r.unit.representation.clone();
                                Self::rep_edit(&mut rep, &context, ui, world);
                                if rep != rm(world).unit.representation {
                                    rm(world).unit.representation = rep;
                                    Self::refresh(world);
                                }
                            }
                        });
                })
                .transparent()
                .pinned()
                .stretch_max()
                .push(world);
            }
            Mode::Representation => {}
            Mode::Vfx => {}
        }
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                if EnumSwitcher::show(&mut rm(world).mode, ui) {
                    Self::load_mode(world);
                }
                if Slider::new("scale").log().ui(
                    &mut world.resource_mut::<CameraData>().cur_scale,
                    5.0..=50.0,
                    ui,
                ) {
                    CameraPlugin::apply(world);
                    Self::load_mode(world);
                }
            });
        })
        .pinned()
        .no_expand()
        .transparent()
        .keep()
        .push(world);
        Self::load_mode(world);
    }
    fn unit_edit(unit: &mut PackedUnit, ui: &mut Ui) {
        Input::new("name:").ui_string(&mut unit.name, ui);
        ui.horizontal(|ui| {
            DragValue::new(&mut unit.pwr).prefix("pwr:").ui(ui);
            DragValue::new(&mut unit.hp).prefix("hp:").ui(ui);
            DragValue::new(&mut unit.lvl).prefix("lvl:").ui(ui);
        });
        ui.horizontal(|ui| {
            let mut rarity: Rarity = unit.rarity.into();
            if Selector::new("rarity").ui_enum(&mut rarity, ui) {
                unit.rarity = rarity.into();
            }
            if let Some(house) = unit.houses.get_mut(0) {
                Selector::new("house").ui_iter(house, game_assets().houses.keys(), ui);
            }
        });
    }
    fn trigger_edit(trigger: &mut Trigger, context: &Context, ui: &mut Ui, world: &mut World) {
        trigger.show_node("", &context, world, ui);
    }
    fn rep_edit(rep: &mut Representation, context: &Context, ui: &mut Ui, world: &mut World) {
        rep.show_node("", context, world, ui);
    }
}
