use spacetimedb_sdk::Table;

use super::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), Self::on_enter)
            .add_systems(OnExit(GameState::Editor), Self::on_exit)
            .add_systems(Update, Self::update.run_if(in_state(GameState::Editor)));
    }
}

#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub struct EditorResource {
    mode: Mode,

    pub battle: (PackedTeam, PackedTeam),

    unit: PackedUnit,
    unit_entity: Option<Entity>,
    unit_mode: UnitMode,
    unit_to_load: String,
    unit_source: Option<(Faction, usize)>,

    #[serde(skip)]
    incubator_link: Option<u64>,
    incubator_card_before: Option<UnitCard>,
    incubator_card: UnitCard,

    representation: Representation,
    vfx: Vfx,
    vfx_entity: Option<Entity>,
    vfx_selected: String,
}
fn rm(world: &mut World) -> Mut<EditorResource> {
    world.resource_mut::<EditorResource>()
}

#[derive(
    Default, EnumIter, AsRefStr, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Debug,
)]
enum Mode {
    #[default]
    Unit,
    Battle,
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

impl ToCstr for Mode {
    fn cstr(&self) -> Cstr {
        self.as_ref().into()
    }
}
impl ToCstr for UnitMode {
    fn cstr(&self) -> Cstr {
        self.as_ref().into()
    }
}

impl EditorPlugin {
    pub fn load_battle(left: PackedTeam, right: PackedTeam, world: &mut World) {
        let mut cs = client_state().clone();
        cs.editor.battle.0 = left;
        cs.editor.battle.1 = right;
        cs.editor.mode = Mode::Battle;
        cs.save();
        Self::load_state(world);
    }
    fn load_state(world: &mut World) {
        world.insert_resource(client_state().editor.clone());
    }
    fn on_enter(world: &mut World) {
        if !world.is_resource_added::<EditorResource>() {
            Self::load_state(world);
        }
    }
    fn on_exit(world: &mut World) {
        rm(world).unit_source = None;
        Self::save_state(world);
        Self::clear(world);
    }
    fn clear(world: &mut World) {
        world.game_clear();
        TilePlugin::clear(world);
    }
    fn save_state(world: &mut World) {
        debug!("Save editor state");
        let mut cs = client_state().clone();
        cs.editor = rm(world).clone();
        cs.save();
    }
    fn refresh(world: &mut World) {
        // world.game_clear();
        for unit in UnitPlugin::collect_faction(Faction::Team, world) {
            world.entity_mut(unit).despawn_recursive();
        }
        match rm(world).mode {
            Mode::Battle => {
                rm(world).battle.0.clone().unpack(Faction::Left, world);
                rm(world).battle.1.clone().unpack(Faction::Right, world);
                UnitPlugin::place_into_slots(world);
            }
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
            Mode::Vfx => {
                rm(world).vfx_entity = rm(world).vfx.clone().unpack(world).ok();
            }
        }
    }
    fn get_team_mut(faction: Faction, r: &mut EditorResource) -> &mut PackedTeam {
        match r.mode {
            Mode::Battle => {
                if faction == Faction::Right {
                    &mut r.battle.1
                } else {
                    &mut r.battle.0
                }
            }
            Mode::Unit | Mode::Vfx => panic!(),
        }
    }
    fn get_team_unit(slot: usize, faction: Faction, world: &mut World) -> Option<PackedUnit> {
        Self::get_team_mut(faction, &mut rm(world))
            .units
            .get(slot)
            .cloned()
    }
    fn set_team_unit(slot: usize, faction: Faction, unit: PackedUnit, world: &mut World) {
        let mut r = rm(world);
        let team = Self::get_team_mut(faction, &mut r);
        if team.units.len() > slot {
            team.units.remove(slot);
            team.units.insert(slot, unit);
        } else {
            team.units.push(unit);
        }
    }
    fn remove_team_unit(slot: usize, faction: Faction, world: &mut World) {
        let mut r = rm(world);
        let team = Self::get_team_mut(faction, &mut r);
        if team.units.len() < slot {
            return;
        }
        team.units.remove(slot);
    }
    fn team_container(faction: Faction) -> TeamContainer {
        TeamContainer::new(faction)
            .top_content(move |ui, world| {
                ui.horizontal(|ui| {
                    if Button::new("Load own").ui(ui).clicked() {
                        Confirmation::new("Open own team")
                            .content(move |ui, world| {
                                cn().db
                                    .team()
                                    .iter()
                                    .filter(|t| {
                                        t.pool.eq(&TeamPool::Owned) && t.owner == player_id()
                                    })
                                    .collect_vec()
                                    .show_modified_table("Teams", ui, world, move |t| {
                                        t.column_btn_dyn(
                                            "select",
                                            Box::new(move |t: &TTeam, _, world| {
                                                let mut r = rm(world);
                                                *Self::get_team_mut(faction, &mut r) =
                                                    PackedTeam::from_id(t.id);
                                                Confirmation::close_current(world);
                                                Self::load_mode(world);
                                            }),
                                        )
                                    });
                            })
                            .cancel(|_| {})
                            .push(world);
                    }

                    ui.add_space(30.0);
                    if Button::new("Paste").ui(ui).clicked() {
                        if let Some(team) = paste_from_clipboard(world)
                            .and_then(|t| ron::from_str::<PackedTeam>(&t).ok())
                        {
                            let mut r = rm(world);
                            *Self::get_team_mut(faction, &mut r) = team.apply_limit();
                            Self::load_mode(world);
                        }
                    }

                    const GAME_MODE_ID: &str = "champion_mode";
                    fn selected_mode(ctx: &egui::Context) -> GameMode {
                        ctx.data(|r| r.get_temp::<GameMode>(Id::new(GAME_MODE_ID)))
                            .unwrap_or_default()
                    }
                    ui.add_space(30.0);
                    if Button::new("Load Champion").ui(ui).clicked() {
                        Confirmation::new("Load Champion")
                            .content(|ui, _| {
                                let mut mode = selected_mode(ui.ctx());
                                if Selector::new("mode").ui_iter(
                                    &mut mode,
                                    [
                                        &GameMode::ArenaNormal,
                                        &GameMode::ArenaRanked,
                                        &GameMode::ArenaConst,
                                    ],
                                    ui,
                                ) {
                                    ui.ctx().data_mut(|w| {
                                        w.insert_temp(Id::new(GAME_MODE_ID), mode);
                                    })
                                }
                            })
                            .accept(move |world| {
                                let ctx = egui_context(world).unwrap();
                                let mode = selected_mode(&ctx);
                                if let Some(champion) = cn()
                                    .db
                                    .arena_leaderboard()
                                    .iter()
                                    .filter(|a| {
                                        a.season == global_settings().season && a.mode == mode
                                    })
                                    .max_by_key(|a| a.floor)
                                {
                                    let mut r = rm(world);
                                    *Self::get_team_mut(faction, &mut r) =
                                        PackedTeam::from_id(champion.team);
                                    Self::load_mode(world);
                                }
                            })
                            .cancel(|_| {})
                            .push(world);
                    }
                });
            })
            .slot_content(move |slot, e, ui, world| {
                if e.is_none() {
                    return;
                }
                if Button::new("Edit").ui(ui).clicked() {
                    if let Some(unit) = Self::get_team_unit(slot, faction, world) {
                        let mut rm = rm(world);
                        rm.unit = unit;
                        rm.mode = Mode::Unit;
                        rm.unit_source = Some((faction, slot));
                        Self::load_mode(world);
                    }
                }
            })
            .context_menu(move |slot, entity, ui, world| {
                ui.reset_style();
                ui.set_min_width(150.0);
                if Button::new("Paste").ui(ui).clicked() {
                    if let Some(v) = paste_from_clipboard(world) {
                        match ron::from_str::<PackedUnit>(&v) {
                            Ok(v) => {
                                Self::set_team_unit(slot, faction, v, world);
                                Self::refresh(world);
                            }
                            Err(e) => format!("Failed to deserialize unit from {v}: {e}")
                                .notify_error(world),
                        }
                    } else {
                        "Clipboard is empty".notify_error(world);
                    }
                    ui.close_menu();
                }
                if entity.is_none() {
                    if Button::new("Spawn default").ui(ui).clicked() {
                        Self::set_team_unit(slot, faction, default(), world);
                        Self::refresh(world);
                        ui.close_menu();
                    }
                } else {
                    if Button::new("Copy").ui(ui).clicked() {
                        if let Some(unit) = Self::get_team_unit(slot, faction, world) {
                            copy_to_clipboard(&unit.to_ron_str(), world);
                        }
                        ui.close_menu();
                    }
                    if Button::new("Delete").red(ui).ui(ui).clicked() {
                        Self::remove_team_unit(slot, faction, world);
                        Self::refresh(world);
                        ui.close_menu();
                    }
                }
            })
            .on_swap(move |slot, target, world| {
                let mut r = rm(world);
                let team = Self::get_team_mut(faction, &mut r);
                let unit = team.units.remove(slot);
                team.units.insert(target.at_most(team.units.len()), unit);
                Self::refresh(world);
            })
    }
    fn load_mode(world: &mut World) {
        let r = rm(world);
        let mode = r.mode;
        info!("Load editor mode {mode}");
        if !matches!(mode, Mode::Unit) {
            if let Some((faction, slot)) = r.unit_source {
                Self::set_team_unit(slot, faction, r.unit.clone(), world);
            }
        }
        Self::clear(world);
        Self::save_state(world);
        match mode {
            Mode::Battle => {
                let b = rm(world).battle.clone();
                b.0.unpack(Faction::Left, world);
                b.1.unpack(Faction::Right, world);
                Tile::new(Side::Top, |ui, world| {
                    ui.columns(2, |ui| {
                        ui[0].vertical(|ui| {
                            ui.push_id("left", |ui| {
                                Self::team_container(Faction::Left).ui(ui, world);
                            });
                        });
                        ui[1].vertical(|ui| {
                            ui.push_id("right", |ui| {
                                Self::team_container(Faction::Right)
                                    .left_to_right()
                                    .ui(ui, world);
                            });
                        });
                    });
                })
                .stretch_min()
                .transparent()
                .pinned()
                .push(world);
                Tile::new(Side::Top, |ui, world| {
                    ui.horizontal(|ui| {
                        if Button::new("Run battle").ui(ui).clicked() {
                            BattlePlugin::load_teams(
                                rm(world).battle.0.clone(),
                                rm(world).battle.1.clone(),
                                world,
                            );
                            BattlePlugin::set_next_state(GameState::Editor, world);
                            GameState::Battle.set_next(world);
                        }
                        if Button::new("Send BattleStart").ui(ui).clicked() {
                            Event::BattleStart.send(world);
                        }
                        if Button::new("Strike").ui(ui).clicked() {
                            if let Some((left, right)) = BattlePlugin::get_strikers(world) {
                                let _ = BattlePlugin::run_strike(left, right, world);
                            }
                        }
                    });
                })
                .transparent()
                .no_expand()
                .pinned()
                .push(world);
            }
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
                                            if Button::new("Copy").ui(ui).clicked() {
                                                copy_to_clipboard(
                                                    &rm(world).unit.clone().to_ron_str(),
                                                    world,
                                                );
                                            }
                                            if Button::new("Paste").ui(ui).clicked() {
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
                let mut r = rm(world);
                if r.unit_to_load.is_empty() {
                    r.unit_to_load = game_assets()
                        .heroes
                        .keys()
                        .choose(&mut thread_rng())
                        .cloned()
                        .unwrap();
                }
                Tile::new(Side::Top, |ui, world| {
                    ui.horizontal(|ui| {
                        let mut r = rm(world);
                        EnumSwitcher::new().show(&mut r.unit_mode, ui);
                        ui.add_space(30.0);
                        Selector::new("").ui_iter(
                            &mut r.unit_to_load,
                            game_assets().heroes.keys().sorted(),
                            ui,
                        );
                        if Button::new("load").ui(ui).clicked() {
                            r.unit = game_assets().heroes.get(&r.unit_to_load).unwrap().clone();
                            Self::refresh(world);
                        }
                        let mut r = rm(world);
                        if let Some((faction, slot)) = r.unit_source {
                            format!("editing {faction} team {slot} unit")
                                .cstr_c(YELLOW)
                                .label(ui);
                            if Button::new("unlink").ui(ui).clicked() {
                                r.unit_source = None;
                            }
                        }
                    });
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
                                Self::unit_edit(&mut unit, ui, world);
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
            Mode::Vfx => {
                Self::refresh(world);
                Tile::new(Side::Top, |ui, world| {
                    ui.horizontal(|ui| {
                        Selector::new("").ui_iter(
                            &mut rm(world).vfx_selected,
                            game_assets().vfxs.keys(),
                            ui,
                        );
                        if Button::new("Load").ui(ui).clicked() {
                            rm(world).vfx = Vfx::get(&rm(world).vfx_selected);
                        }
                        if Button::new("Copy").ui(ui).clicked() {
                            copy_to_clipboard(&ron::to_string(&rm(world).vfx).unwrap(), world);
                        }
                        if Button::new("Paste").ui(ui).clicked() {
                            if let Some(s) = paste_from_clipboard(world) {
                                match ron::from_str(&s) {
                                    Ok(v) => {
                                        rm(world).vfx = v;
                                        Self::refresh(world);
                                    }
                                    Err(e) => format!("Clipboard deserialize error: {e}")
                                        .notify_error(world),
                                }
                            } else {
                                "Clipboard is empty".notify_error(world);
                            }
                        }
                    });
                })
                .pinned()
                .no_expand()
                .transparent()
                .push(world);
                Tile::new(Side::Left, |ui, world| {
                    ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let r = rm(world);
                            let Some(entity) = r.vfx_entity else {
                                return;
                            };
                            let mut vfx = r.vfx.clone();
                            vfx.show_node(
                                "",
                                Context::new(entity).set_var(VarName::Index, default()),
                                world,
                                ui,
                            );
                            if vfx != rm(world).vfx {
                                rm(world).vfx = vfx;
                                Self::refresh(world);
                            }
                        });
                })
                .transparent()
                .pinned()
                .stretch_max()
                .push(world);
            }
        }
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                if EnumSwitcher::new().show(&mut rm(world).mode, ui) {
                    Self::load_mode(world);
                }
                ui.add_space(50.0);
                if Slider::new("scale").log().ui(
                    &mut world.resource_mut::<CameraData>().cur_scale,
                    5.0..=50.0,
                    ui,
                ) {
                    CameraPlugin::apply(world);
                    Self::load_mode(world);
                }
                if Button::new("Save state").ui(ui).clicked() {
                    Self::save_state(world);
                }
                if Button::new("Load state").ui(ui).clicked() {
                    Self::load_state(world);
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
    fn unit_edit(unit: &mut PackedUnit, ui: &mut Ui, world: &mut World) {
        let base_unit: TBaseUnit = unit.clone().into();
        let texture = TextureRenderPlugin::texture_base_unit(&base_unit, world);
        ui.image(egui::load::SizedTexture::new(
            texture,
            egui::vec2(256., 256.),
        ));
        ui.horizontal(|ui| {
            for texture in TextureRenderPlugin::textures(world) {
                ui.image(egui::load::SizedTexture::new(
                    texture,
                    egui::vec2(128., 128.),
                ));
            }
        });
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
    fn update(world: &mut World) {
        let r = rm(world);
        match r.mode {
            Mode::Vfx => {
                let Some(entity) = r.vfx_entity else {
                    return;
                };
                if !VarState::try_get(entity, world).is_ok_and(|s| s.is_animating()) {
                    Self::refresh(world);
                }
            }
            Mode::Battle | Mode::Unit => {}
        }
    }
    pub fn load_unit(unit: PackedUnit, world: &mut World) {
        let mut r = rm(world);
        r.incubator_link = None;
        r.unit = unit.into();
        r.mode = Mode::Unit;
    }
}
