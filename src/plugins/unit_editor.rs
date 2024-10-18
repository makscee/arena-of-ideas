use serde::de::DeserializeOwned;

use super::*;

pub struct UnitEditorPlugin;

impl Plugin for UnitEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UnitEditor), Self::on_enter)
            .add_systems(OnExit(GameState::UnitEditor), Self::on_exit)
            .init_resource::<UnitEditorResource>();
    }
}

#[derive(Resource, Default)]
struct UnitEditorResource {
    teams: HashMap<Faction, PackedTeam>,
    editing_hero: PackedUnit,
    editing_entity: Option<Entity>,
    editing_slot: usize,
    editing_faction: Faction,
    hero_to_spawn: String,
    editing_trigger: Trigger,
    editing_representation: Representation,
}
fn rm(world: &mut World) -> Mut<UnitEditorResource> {
    world.resource_mut::<UnitEditorResource>()
}

const UNIT_ID: &str = "unit_edit";
const REP_ID: &str = "rep_edit";
impl UnitEditorPlugin {
    fn on_enter(world: &mut World) {
        Self::load_state(world);
        let mut r = rm(world);
        for faction in [Faction::Left, Faction::Right] {
            if r.teams.contains_key(&faction) {
                continue;
            }
            r.teams.insert(faction, default());
        }
        Self::respawn_teams(false, world);
    }
    fn on_exit(world: &mut World) {
        TeamPlugin::despawn(Faction::Left, world);
        TeamPlugin::despawn(Faction::Right, world);
    }
    fn save_state(world: &mut World) {
        let mut cs = client_state().clone();
        cs.editor_teams = rm(world).teams.clone();
        cs.save();
    }
    fn load_state(world: &mut World) {
        Self::close_editors(world);
        let mut r = rm(world);
        r.teams = client_state().editor_teams.clone();
        Self::respawn_teams(false, world);
    }
    fn apply_edits(world: &mut World) {
        let r = rm(world);
        let unit = r.editing_hero.clone();
        let faction = r.editing_faction;
        let slot = r.editing_slot;
        if let Some(entity) = r.editing_entity {
            UnitPlugin::despawn(entity, world);
        }
        Self::set_team_unit(unit.clone(), faction, slot, world);
        let unit = unit.unpack(
            TeamPlugin::entity(faction, world),
            Some(slot as i32),
            None,
            world,
        );
        UnitPlugin::place_into_slot(unit, world);
        rm(world).editing_entity = Some(unit);
    }
    fn respawn_teams(instant_placement: bool, world: &mut World) {
        TeamPlugin::despawn(Faction::Left, world);
        TeamPlugin::despawn(Faction::Right, world);
        for (faction, team) in rm(world).teams.clone() {
            team.unpack(faction, world);
        }
        if instant_placement {
            UnitPlugin::place_into_slots(world);
        }
    }
    fn set_team_unit(unit: PackedUnit, faction: Faction, slot: usize, world: &mut World) {
        if let Some(team) = rm(world).teams.get_mut(&faction) {
            if team.units.len() <= slot {
                team.units.push(unit);
            } else {
                team.units.remove(slot);
                team.units.insert(slot, unit);
            }
        }
    }
    fn remove_team_unit(faction: Faction, slot: usize, world: &mut World) {
        if let Some(team) = rm(world).teams.get_mut(&faction) {
            team.units.remove(slot);
            UnitEditorPlugin::respawn_teams(true, world);
        }
    }
    fn move_team_units(faction: Faction, from: usize, to: usize, world: &mut World) {
        if let Some(team) = rm(world).teams.get_mut(&faction) {
            if from >= team.units.len() {
                format!("Wrong slot index").notify_error(world);
                return;
            }
            let unit = team.units.remove(from);
            team.units.insert(to.at_most(team.units.len()), unit);
            Self::respawn_teams(false, world);
        }
    }
    pub fn load_team(faction: Faction, team: PackedTeam, world: &mut World) {
        rm(world).teams.insert(faction, team);
    }
    fn open_new_unit_picker(faction: Faction, slot: usize, ctx: &egui::Context, world: &mut World) {
        let mut r = rm(world);
        r.editing_faction = faction;
        r.editing_slot = slot;
        Confirmation::new("Add Unit".cstr())
            .accept(|world| {
                let r = rm(world);
                let unit = TBaseUnit::find_by_name(r.hero_to_spawn.clone())
                    .map(|u| PackedUnit::from(u))
                    .unwrap_or_default();
                Self::set_team_unit(unit, r.editing_faction, r.editing_slot, world);
                Self::respawn_teams(true, world);
            })
            .cancel(|_| {})
            .content(|ui, world| {
                if Button::click("Spawn Default").ui(ui).clicked() {
                    let mut r = rm(world);
                    let faction = r.editing_faction;
                    r.teams
                        .get_mut(&faction)
                        .unwrap()
                        .units
                        .push(PackedUnit::default());
                    Self::respawn_teams(true, world);
                    Confirmation::close_current(ui.ctx());
                }
                let mut r = rm(world);
                Selector::new("spawn").ui_iter(
                    &mut r.hero_to_spawn,
                    &TBaseUnit::iter()
                        .filter_map(|u| match u.rarity >= 0 {
                            true => Some(u.name),
                            false => None,
                        })
                        .collect_vec(),
                    ui,
                );
            })
            .push(ctx);
    }
    fn load_unit_editor(entity: Entity, faction: Faction, slot: usize, world: &mut World) {
        let mut r = rm(world);
        r.editing_faction = faction;
        r.editing_slot = slot;
        r.editing_entity = Some(entity);
        r.editing_hero = r
            .teams
            .get(&faction)
            .and_then(|t| t.units.get(slot))
            .cloned()
            .unwrap_or_default();
    }
    fn editor_open(world: &mut World) -> bool {
        TilePlugin::is_open(UNIT_ID, world) || TilePlugin::is_open(REP_ID, world)
    }
    fn close_editors(world: &mut World) {
        if Self::editor_open(world) {
            TilePlugin::close(UNIT_ID, world);
            TilePlugin::close(REP_ID, world);
        }
        let ctx = &egui_context(world).unwrap();
        while Confirmation::has_active(ctx) {
            Confirmation::close_current(ctx);
        }
    }
    fn open_unit_editor(world: &mut World) {
        Self::close_editors(world);
        Tile::new(Side::Left, |ui, world| {
            let mut hero = rm(world).editing_hero.clone();
            Input::new("name:").ui_string(&mut hero.name, ui);
            ui.horizontal(|ui| {
                DragValue::new(&mut hero.pwr).prefix("pwr:").ui(ui);
                DragValue::new(&mut hero.hp).prefix("hp:").ui(ui);
                DragValue::new(&mut hero.lvl).prefix("lvl:").ui(ui);
            });
            ui.horizontal(|ui| {
                let mut rarity: Rarity = hero.rarity.into();
                if Selector::new("rarity").ui_enum(&mut rarity, ui) {
                    hero.rarity = rarity.into();
                }
                if let Some(house) = hero.houses.get_mut(0) {
                    Selector::new("house").ui_iter(house, game_assets().houses.keys(), ui);
                }
            });
            let trigger = hero.trigger.cstr_expanded();
            let mut r = rm(world);
            if !r.editing_hero.eq(&hero) {
                r.editing_hero = hero;
                Self::apply_edits(world);
            }
            ui.horizontal(|ui| {
                "trigger:".cstr().label(ui);
                if trigger.button(ui).clicked() {
                    let mut r = rm(world);
                    r.editing_trigger = r.editing_hero.trigger.clone();
                    Confirmation::new("Trigger Editor".cstr())
                        .accept(|world| {
                            let mut r = rm(world);
                            r.editing_hero.trigger = mem::take(&mut r.editing_trigger);
                            Self::apply_edits(world);
                        })
                        .cancel(|_| {})
                        .content(|ui, world| {
                            let mut r = rm(world);
                            let context = Context::new(r.editing_entity.unwrap());
                            let mut trigger = mem::take(&mut r.editing_trigger);
                            trigger.show_node("", &context, world, ui);
                            rm(world).editing_trigger = trigger;
                        })
                        .push(ui.ctx());
                }
            });
            if Button::click("Edit representation").ui(ui).clicked() {
                Confirmation::close_current(ui.ctx());
                let mut r = rm(world);
                r.editing_representation = r.editing_hero.representation.clone();
                Tile::new(Side::Left, |ui, world| {
                    ScrollArea::both().show(ui, |ui| {
                        let r = rm(world);
                        let mut repr = r.editing_representation.clone();
                        let context = Context::new(r.editing_entity.unwrap())
                            .set_var(VarName::Index, 1.into())
                            .take();
                        repr.show_node("", &context, world, ui);
                        let mut r = rm(world);
                        if !repr.eq(&r.editing_representation) {
                            r.editing_hero.representation = repr.clone();
                            r.editing_representation = repr;
                            Self::apply_edits(world);
                        }
                    });
                })
                .with_id(REP_ID.into())
                .transparent()
                .stretch_max()
                .push(world);
            }
        })
        .with_id(UNIT_ID.into())
        .non_focusable()
        .push(world);
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                if Button::click("Reload").ui(ui).clicked() {
                    Self::load_state(world);
                }
                if Button::click("Save").ui(ui).clicked() {
                    Self::save_state(world);
                }
                if Button::click("Load sample").ui(ui).clicked() {
                    for faction in [Faction::Left, Faction::Right] {
                        Self::load_team(
                            faction,
                            PackedTeam {
                                units: [default()].into(),
                                ..default()
                            },
                            world,
                        );
                    }

                    Self::respawn_teams(true, world);
                }
                if Button::click("Run battle").ui(ui).clicked() {
                    BattlePlugin::load_teams(
                        rm(world).teams.get(&Faction::Left).unwrap().clone(),
                        rm(world).teams.get(&Faction::Right).unwrap().clone(),
                        world,
                    );
                    BattlePlugin::set_next_state(GameState::UnitEditor, world);
                    GameState::Battle.proceed_to_target(world);
                }
                if Button::click("Strike").ui(ui).clicked() {
                    if let Some((left, right)) = BattlePlugin::get_strikers(world) {
                        let _ = BattlePlugin::run_strike(left, right, world);
                    }
                }
            });
        })
        .pinned()
        .transparent()
        .non_focusable()
        .stretch_min()
        .push(world);
        Tile::new(Side::Top, |ui, world| {
            if ui.available_width() < 10.0 {
                return;
            }
            fn on_click(slot: usize, faction: Faction, entity: Option<Entity>, world: &mut World) {
                let ctx = &egui_context(world).unwrap();
                if Confirmation::has_active(ctx) {
                    return;
                }
                if let Some(entity) = entity {
                    UnitEditorPlugin::load_unit_editor(entity, faction, slot, world);
                    UnitEditorPlugin::open_unit_editor(world);
                } else {
                    UnitEditorPlugin::open_new_unit_picker(faction, slot, ctx, world);
                }
            }
            fn context_menu(
                slot: usize,
                faction: Faction,
                entity: Option<Entity>,
                ui: &mut Ui,
                world: &mut World,
            ) {
                ui.reset_style();
                ui.set_min_width(150.0);
                if Button::click("Paste").ui(ui).clicked() {
                    if let Some(v) = paste_from_clipboard(world) {
                        match ron::from_str::<PackedUnit>(&v) {
                            Ok(v) => {
                                UnitEditorPlugin::set_team_unit(v, faction, slot, world);
                                UnitEditorPlugin::respawn_teams(true, world)
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
                    if Button::click("Spawn default").red(ui).ui(ui).clicked() {
                        UnitEditorPlugin::set_team_unit(default(), faction, slot, world);
                        UnitEditorPlugin::respawn_teams(true, world);
                        ui.close_menu();
                    }
                } else {
                    if Button::click("Copy").ui(ui).clicked() {
                        if let Some(unit) = rm(world)
                            .teams
                            .get(&faction)
                            .and_then(|t| t.units.get(slot))
                        {
                            match ron::to_string(unit) {
                                Ok(v) => copy_to_clipboard(&v, world),
                                Err(e) => {
                                    format!("Failed to serialize unit: {e}").notify_error(world)
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if Button::click("Delete").red(ui).ui(ui).clicked() {
                        UnitEditorPlugin::remove_team_unit(faction, slot, world);
                        ui.close_menu();
                    }
                }
            }
            let (slot, faction) = {
                if Self::editor_open(world) {
                    let r = rm(world);
                    (Some(r.editing_slot), r.editing_faction)
                } else {
                    (None, Faction::Team)
                }
            };
            ui.columns(2, |ui| {
                TeamContainer::new(Faction::Left)
                    .on_click(|slot, entity, world| on_click(slot, Faction::Left, entity, world))
                    .context_menu(|slot, entity, ui, world| {
                        context_menu(slot, Faction::Left, entity, ui, world)
                    })
                    .on_swap(|from, to, world| {
                        UnitEditorPlugin::move_team_units(Faction::Left, from, to, world);
                    })
                    .highlighted_slot(if faction == Faction::Left { slot } else { None })
                    .ui(&mut ui[0], world);
                TeamContainer::new(Faction::Right)
                    .left_to_right()
                    .on_click(|slot, entity, world| on_click(slot, Faction::Right, entity, world))
                    .context_menu(|slot, entity, ui, world| {
                        context_menu(slot, Faction::Right, entity, ui, world)
                    })
                    .on_swap(|from, to, world| {
                        UnitEditorPlugin::move_team_units(Faction::Left, from, to, world);
                    })
                    .highlighted_slot(if faction == Faction::Right {
                        slot
                    } else {
                        None
                    })
                    .ui(&mut ui[1], world);
            });
        })
        .stretch_min()
        .transparent()
        .pinned()
        .push(world);
    }
}

fn lookup_id() -> Id {
    static LOOKUP_STRING: OnceCell<Id> = OnceCell::new();
    *LOOKUP_STRING.get_or_init(|| Id::new("lookup_string"))
}
fn lookup_text(ctx: &egui::Context) -> String {
    ctx.data(|r| r.get_temp::<String>(lookup_id()).unwrap_or_default())
}
fn lookup_text_clear(ctx: &egui::Context) {
    ctx.data_mut(|w| w.remove_temp::<String>(lookup_id()));
}
fn lookup_text_push(ctx: &egui::Context, s: &str) {
    ctx.data_mut(|w| w.get_temp_mut_or_default::<String>(lookup_id()).push_str(s));
}
fn lookup_text_pop(ctx: &egui::Context) {
    ctx.data_mut(|w| w.get_temp_mut_or_default::<String>(lookup_id()).pop());
}
pub trait ShowEditor: ToCstr + Default + Serialize + DeserializeOwned + Clone {
    fn transparent() -> bool {
        false
    }
    fn get_variants() -> impl Iterator<Item = Self>;
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>>;
    fn wrapper() -> Option<Self> {
        None
    }
    fn show_content(&mut self, _context: &Context, _world: &mut World, _ui: &mut Ui) {}
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui);
    fn show_node(&mut self, name: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        const SHADOW: Shadow = Shadow {
            offset: egui::Vec2::ZERO,
            blur: 5.0,
            spread: 5.0,
            color: Color32::from_rgba_premultiplied(20, 20, 20, 25),
        };
        const FRAME_REGULAR: Frame = Frame {
            inner_margin: Margin::same(4.0),
            rounding: Rounding::same(6.0),
            shadow: SHADOW,
            outer_margin: Margin::ZERO,
            fill: BG_DARK,
            stroke: Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            },
        };
        const FRAME_TRANSPARENT: Frame = Frame {
            inner_margin: Margin::same(4.0),
            rounding: Rounding::same(6.0),
            shadow: Shadow::NONE,
            outer_margin: Margin::ZERO,
            fill: TRANSPARENT,
            stroke: Stroke {
                width: 1.0,
                color: VISIBLE_DARK,
            },
        };
        if !name.is_empty() {
            name.cstr_cs(VISIBLE_DARK, CstrStyle::Small).label(ui);
        }
        let resp = if Self::transparent() {
            FRAME_TRANSPARENT
        } else {
            FRAME_REGULAR
        }
        .show(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.vertical(|ui| {
                    self.show_self(ui, world);
                    ui.set_max_width(ui.min_size().x);
                    self.show_content(context, world, ui);
                });
                ui.vertical(|ui| {
                    self.show_children(context, world, ui);
                });
            });
        });
        if let Some(mut wrapper) = Self::wrapper() {
            let rect = resp.response.rect;
            let rect =
                Rect::from_center_size(rect.left_center(), egui::vec2(2.0, rect.height() - 13.0));
            let e_rect = rect.expand2(egui::vec2(4.0, 0.0));
            let ui = &mut ui.child_ui(rect, *ui.layout(), None);
            let resp = ui.allocate_rect(e_rect, Sense::click());
            let color = if resp.hovered() {
                YELLOW
            } else {
                VISIBLE_LIGHT
            };
            ui.painter().rect_filled(rect, Rounding::ZERO, color);
            if resp.clicked() {
                let mut inner = wrapper.get_inner_mut();
                if inner.is_empty() {
                    return;
                }
                *inner[0] = Box::new(self.clone());
                *self = wrapper;
            }
        }
    }
    fn show_self(&mut self, ui: &mut Ui, world: &mut World) {
        let widget = self.cstr().widget(1.0, ui);
        let variants = Self::get_variants();
        let resp = if variants.try_len().is_ok_and(|l| l > 0) {
            let resp = ui
                .menu_button(widget, |ui| {
                    lookup_text(ui.ctx()).cstr().label(ui);
                    let mut take_first = false;
                    for e in ui.ctx().input(|i| i.events.clone()) {
                        match e {
                            egui::Event::Text(s) => lookup_text_push(ui.ctx(), &s),
                            egui::Event::Key {
                                key: Key::Backspace,
                                pressed: true,
                                ..
                            } => lookup_text_pop(ui.ctx()),
                            egui::Event::Key {
                                key: Key::Tab,
                                pressed: true,
                                ..
                            } => take_first = true,
                            _ => {}
                        }
                    }
                    ui.set_min_height(300.0);
                    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        ui.reset_style();
                        let lookup = lookup_text(ui.ctx()).to_lowercase();
                        for e in Self::get_variants() {
                            let c = e.cstr();
                            if !lookup.is_empty()
                                && !c.get_text().to_lowercase().starts_with(&lookup)
                            {
                                continue;
                            }
                            if c.button(ui).clicked() || take_first {
                                self.replace_self(e);
                                ui.close_menu();
                                break;
                            }
                        }
                    });
                })
                .response;
            if resp.clicked() {
                lookup_text_clear(ui.ctx());
            }
            resp
        } else {
            ui.label(widget)
        };
        resp.context_menu(|ui| {
            ui.reset_style();
            if Button::click("Copy").ui(ui).clicked() {
                match ron::to_string(self) {
                    Ok(v) => {
                        copy_to_clipboard(&v, world);
                    }
                    Err(e) => format!("Failed to copy: {e}").notify_error(world),
                }
                ui.close_menu();
            }
            if Button::click("Paste").ui(ui).clicked() {
                if let Some(v) = paste_from_clipboard(world) {
                    match ron::from_str::<Self>(&v) {
                        Ok(v) => *self = v,
                        Err(e) => format!("Failed to paste text {v}: {e}").notify_error(world),
                    }
                } else {
                    format!("Clipboard is empty").notify_error(world);
                }
                ui.close_menu();
            }
        });
    }
    fn replace_self(&mut self, value: Self) {
        let mut inner = self
            .get_inner_mut()
            .into_iter()
            .map(|i| mem::take(i))
            .rev()
            .collect_vec();
        *self = value;
        for s in self.get_inner_mut() {
            if let Some(i) = inner.pop() {
                *s = i;
            } else {
                break;
            }
        }
    }
}
