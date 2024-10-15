use egui::{Checkbox, DragValue, Key, ScrollArea};
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
}
fn rm(world: &mut World) -> Mut<UnitEditorResource> {
    world.resource_mut::<UnitEditorResource>()
}

impl UnitEditorPlugin {
    fn on_enter(world: &mut World) {
        let mut r = rm(world);
        r.teams = client_state().editor_teams.clone();
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
    fn respawn_teams(instant_placement: bool, world: &mut World) {
        TeamPlugin::despawn(Faction::Left, world);
        TeamPlugin::despawn(Faction::Right, world);
        for (faction, team) in rm(world).teams.clone() {
            team.unpack(faction, world);
        }
        if instant_placement {
            UnitPlugin::place_into_slots(world);
        }
        let mut cs = client_state().clone();
        cs.editor_teams = rm(world).teams.clone();
        cs.save();
    }
    fn set_team_unit(unit: PackedUnit, faction: Faction, slot: usize, world: &mut World) {
        let mut r = rm(world);
        if let Some(team) = rm(world).teams.get_mut(&faction) {
            if team.units.len() <= slot {
                team.units.push(unit);
            } else {
                team.units.remove(slot);
                team.units.insert(slot, unit);
            }
            UnitEditorPlugin::respawn_teams(true, world);
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
        Confirmation::new("Add Unit".cstr(), |world| {
            let r = rm(world);
            let unit = TBaseUnit::find_by_name(r.hero_to_spawn.clone())
                .map(|u| PackedUnit::from(u))
                .unwrap_or_default();
            Self::set_team_unit(unit, r.editing_faction, r.editing_slot, world);
        })
        .content(|ui, world| {
            if Button::click("Spawn Default".into()).ui(ui).clicked() {
                let mut r = rm(world);
                let faction = r.editing_faction;
                r.teams
                    .get_mut(&faction)
                    .unwrap()
                    .units
                    .push(PackedUnit::default());
                Self::respawn_teams(true, world);
                Confirmation::pop(ui.ctx());
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
    fn open_unit_editor(ctx: &egui::Context) {
        Confirmation::new(
            "Edit Unit".cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
            |world| {
                let mut r = rm(world);
                let unit = mem::take(&mut r.editing_hero);
                Self::set_team_unit(unit, r.editing_faction, r.editing_slot, world);
            },
        )
        .content(|ui, world| {
            let hero = &mut rm(world).editing_hero;
            Input::new("name:").ui_string(&mut hero.name, ui);
            ui.columns(2, |ui| {
                DragValue::new(&mut hero.pwr).prefix("pwr:").ui(&mut ui[0]);
                DragValue::new(&mut hero.hp).prefix("hp:").ui(&mut ui[1]);
            });
            let trigger = hero.trigger.cstr_expanded();
            ui.horizontal(|ui| {
                "trigger:".cstr().label(ui);
                if trigger.button(ui).clicked() {
                    let mut r = rm(world);
                    r.editing_trigger = r.editing_hero.trigger.clone();
                    Confirmation::new("Trigger Editor".cstr(), |world| {
                        let mut r = rm(world);
                        r.editing_hero.trigger = mem::take(&mut r.editing_trigger);
                    })
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
        })
        .push(ctx);
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Top, |ui, world| {
            ui.horizontal(|ui| {
                if Button::click("Load sample".into()).ui(ui).clicked() {
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
                if Button::click("Strike".into()).ui(ui).clicked() {
                    if let Some((left, right)) = BattlePlugin::get_strikers(world) {
                        let _ = BattlePlugin::run_strike(left, right, world);
                    }
                }
            });
        })
        .pinned()
        .transparent()
        .non_focusable()
        .push(world);
        Tile::new(Side::Top, |ui, world| {
            if ui.available_width() < 10.0 {
                return;
            }
            ui.add_space(100.0);
            fn on_click(slot: usize, faction: Faction, entity: Option<Entity>, world: &mut World) {
                let ctx = &egui_context(world).unwrap();
                if Confirmation::has_active(ctx) {
                    return;
                }
                if let Some(entity) = entity {
                    UnitEditorPlugin::load_unit_editor(entity, faction, slot, world);
                    UnitEditorPlugin::open_unit_editor(ctx);
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
                if Button::click("Paste".into()).ui(ui).clicked() {
                    if let Some(v) = get_from_clipboard(world) {
                        match ron::from_str::<PackedUnit>(&v) {
                            Ok(v) => UnitEditorPlugin::set_team_unit(v, faction, slot, world),
                            Err(e) => format!("Failed to deserialize unit from {v}: {e}")
                                .notify_error(world),
                        }
                    } else {
                        "Clipboard is empty".notify_error(world);
                    }
                    ui.close_menu();
                }
                if entity.is_none() {
                    if Button::click("Spawn default".into())
                        .red(ui)
                        .ui(ui)
                        .clicked()
                    {
                        UnitEditorPlugin::set_team_unit(default(), faction, slot, world);
                        ui.close_menu();
                    }
                } else {
                    if Button::click("Copy".into()).ui(ui).clicked() {
                        if let Some(unit) = rm(world)
                            .teams
                            .get(&faction)
                            .and_then(|t| t.units.get(slot))
                        {
                            match ron::to_string(unit) {
                                Ok(v) => save_to_clipboard(&v, world),
                                Err(e) => {
                                    format!("Failed to serialize unit: {e}").notify_error(world)
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if Button::click("Delete".into()).red(ui).ui(ui).clicked() {
                        UnitEditorPlugin::remove_team_unit(faction, slot, world);
                        ui.close_menu();
                    }
                }
            }
            ui.columns(2, |ui| {
                TeamContainer::new(Faction::Left)
                    .right_to_left()
                    .on_click(|slot, entity, world| on_click(slot, Faction::Left, entity, world))
                    .context_menu(|slot, entity, ui, world| {
                        context_menu(slot, Faction::Left, entity, ui, world)
                    })
                    .on_swap(|from, to, world| {
                        UnitEditorPlugin::move_team_units(Faction::Left, from, to, world);
                    })
                    .ui(&mut ui[0], world);
                TeamContainer::new(Faction::Right)
                    .on_click(|slot, entity, world| on_click(slot, Faction::Right, entity, world))
                    .context_menu(|slot, entity, ui, world| {
                        context_menu(slot, Faction::Right, entity, ui, world)
                    })
                    .on_swap(|from, to, world| {
                        UnitEditorPlugin::move_team_units(Faction::Left, from, to, world);
                    })
                    .ui(&mut ui[1], world);
            });
        })
        .max()
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
pub trait ShowEditor: ToCstr + IntoEnumIterator + Default + Serialize + DeserializeOwned {
    fn show_node(&mut self, name: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        const SHADOW: Shadow = Shadow {
            offset: egui::Vec2::ZERO,
            blur: 5.0,
            spread: 5.0,
            color: Color32::from_rgba_premultiplied(20, 20, 20, 25),
        };
        const FRAME: Frame = Frame {
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
        if !name.is_empty() {
            name.cstr_cs(VISIBLE_DARK, CstrStyle::Small).label(ui);
        }
        FRAME.show(ui, |ui| {
            ui.horizontal(|ui| {
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
    }
    fn show_content(&mut self, _context: &Context, _world: &mut World, _ui: &mut Ui) {}
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui);
    fn show_self(&mut self, ui: &mut Ui, world: &mut World) {
        let widget = self.cstr().widget(1.0, ui);
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
                    for e in Self::iter() {
                        let c = e.cstr();
                        if !lookup.is_empty() && !c.get_text().to_lowercase().starts_with(&lookup) {
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
        resp.context_menu(|ui| {
            ui.reset_style();
            if Button::click("Copy".into()).ui(ui).clicked() {
                match ron::to_string(self) {
                    Ok(v) => {
                        save_to_clipboard(&v, world);
                    }
                    Err(e) => format!("Failed to copy: {e}").notify_error(world),
                }
                ui.close_menu();
            }
            if Button::click("Paste".into()).ui(ui).clicked() {
                if let Some(v) = get_from_clipboard(world) {
                    match ron::from_str::<Self>(&v) {
                        Ok(v) => self.replace_self(v),
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
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>>;
}

fn show_named_node<T: ShowEditor>(
    name: &mut Option<String>,
    node: &mut T,
    context: &Context,
    world: &mut World,
    ui: &mut Ui,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if Checkbox::new(&mut name.is_some(), "").ui(ui).changed() {
                if name.is_some() {
                    *name = None;
                } else {
                    *name = Some(default());
                }
            }
            if let Some(name) = name {
                Input::new("rename").ui_string(name, ui);
            }
        });
        ui.horizontal(|ui| {
            node.show_node("", context, world, ui);
        });
    });
}
impl ShowEditor for Trigger {
    fn show_node(&mut self, _: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        self.show_self(ui, world);
        self.cstr_expanded().label(ui);
        self.show_children(context, world, ui);
    }
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                let mut c = 0;
                ui.collapsing("Triggers", |ui| {
                    for (node, name) in triggers.iter_mut() {
                        c += 1;
                        ui.push_id(c, |ui| {
                            show_named_node(name, node, context, world, ui);
                        });
                    }
                    if Button::click("+".into()).ui(ui).clicked() {
                        triggers.push((default(), None));
                    }
                });
                ui.collapsing("Targets", |ui| {
                    for (node, name) in targets.iter_mut() {
                        c += 1;
                        ui.push_id(c, |ui| {
                            show_named_node(name, node, context, world, ui);
                        });
                    }
                    if Button::click("+".into()).ui(ui).clicked() {
                        targets.push((default(), None));
                    }
                });
                ui.collapsing("Effects", |ui| {
                    for (node, name) in effects.iter_mut() {
                        c += 1;
                        ui.push_id(c, |ui| {
                            show_named_node(name, node, context, world, ui);
                        });
                    }
                    if Button::click("+".into()).ui(ui).clicked() {
                        effects.push((default(), None));
                    }
                });
            }
            Trigger::Change { trigger, expr } => todo!(),
            Trigger::List(_) => todo!(),
        }
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        default()
    }
}

impl ShowEditor for Expression {
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits
            | Expression::Value(..)
            | Expression::Context(..)
            | Expression::OwnerState(..)
            | Expression::TargetState(..)
            | Expression::CasterState(..)
            | Expression::OwnerStateLast(..)
            | Expression::TargetStateLast(..)
            | Expression::CasterStateLast(..)
            | Expression::StatusState(_, _)
            | Expression::StatusStateLast(_, _)
            | Expression::AbilityContext(_, _)
            | Expression::AbilityState(_, _)
            | Expression::StatusCharges(_)
            | Expression::HexColor(_)
            | Expression::F(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::S(_)
            | Expression::V2(_, _) => {}
            Expression::FilterStatusUnits(_, e)
            | Expression::FilterNoStatusUnits(_, e)
            | Expression::StatusEntity(_, e)
            | Expression::Dbg(e)
            | Expression::Ctx(e)
            | Expression::ToI(e)
            | Expression::Vec2E(e)
            | Expression::UnitVec(e)
            | Expression::VX(e)
            | Expression::VY(e)
            | Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sqr(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::Floor(e)
            | Expression::Ceil(e)
            | Expression::Fract(e)
            | Expression::SlotUnit(e)
            | Expression::RandomF(e)
            | Expression::RandomUnit(e)
            | Expression::ListCount(e) => e.show_node("", context, world, ui),

            Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b) => {
                a.show_node("", context, world, ui);
                b.show_node("", context, world, ui);
            }

            Expression::MaxUnit(value, units) => {
                value.show_node("value", context, world, ui);
                units.show_node("units", context, world, ui);
            }
            Expression::RandomUnitSubset(amount, units) => {
                amount.show_node("amount", context, world, ui);
                units.show_node("units", context, world, ui);
            }
            Expression::Vec2EE(x, y) => {
                x.show_node("x", context, world, ui);
                y.show_node("y", context, world, ui);
            }
            Expression::WithVar(_, value, expression) => {
                value.show_node("value", context, world, ui);
                expression.show_node("e", context, world, ui);
            }
            Expression::If(cond, th, el) => {
                cond.show_node("condition", context, world, ui);
                th.show_node("then", context, world, ui);
                el.show_node("else", context, world, ui);
            }
        }
    }

    fn show_content(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        let value = self.get_value(context, world);
        match self {
            Expression::FilterStatusUnits(status, _)
            | Expression::FilterNoStatusUnits(status, _)
            | Expression::StatusEntity(status, _) => {
                status_selector(status, world, ui);
            }
            Expression::Value(v) => {
                v.cstr().label(ui);
            }
            Expression::Context(var)
            | Expression::OwnerState(var)
            | Expression::TargetState(var)
            | Expression::CasterState(var)
            | Expression::OwnerStateLast(var)
            | Expression::TargetStateLast(var)
            | Expression::CasterStateLast(var)
            | Expression::WithVar(var, ..) => {
                var_selector(var, ui);
            }
            Expression::StatusState(status, var) | Expression::StatusStateLast(status, var) => {
                status_selector(status, world, ui);
                var_selector(var, ui);
            }
            Expression::AbilityContext(ability, var) | Expression::AbilityState(ability, var) => {
                ability_selector(ability, world, ui);
                var_selector(var, ui);
            }
            Expression::StatusCharges(status) => status_selector(status, world, ui),
            Expression::HexColor(color) => {
                if let Ok(value) = value.as_ref() {
                    if let Ok(mut c32) = value.get_color32() {
                        if ui.color_edit_button_srgba(&mut c32).changed() {
                            *color = c32.to_hex();
                        }
                    }
                }
            }
            Expression::F(v) => {
                DragValue::new(v).ui(ui);
            }
            Expression::I(v) => {
                DragValue::new(v).ui(ui);
            }
            Expression::B(v) => {
                Checkbox::new(v, "").ui(ui);
            }
            Expression::S(v) => {
                Input::new("").ui_string(v, ui);
            }
            Expression::V2(x, y) => {
                DragValue::new(x).ui(ui);
                DragValue::new(y).ui(ui);
            }

            Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits
            | Expression::Dbg(..)
            | Expression::Ctx(..)
            | Expression::ToI(..)
            | Expression::Vec2E(..)
            | Expression::UnitVec(..)
            | Expression::VX(..)
            | Expression::VY(..)
            | Expression::Sin(..)
            | Expression::Cos(..)
            | Expression::Sqr(..)
            | Expression::Even(..)
            | Expression::Abs(..)
            | Expression::Floor(..)
            | Expression::Ceil(..)
            | Expression::Fract(..)
            | Expression::SlotUnit(..)
            | Expression::RandomF(..)
            | Expression::RandomUnit(..)
            | Expression::ListCount(..)
            | Expression::MaxUnit(..)
            | Expression::RandomUnitSubset(..)
            | Expression::Vec2EE(..)
            | Expression::Sum(..)
            | Expression::Sub(..)
            | Expression::Mul(..)
            | Expression::Div(..)
            | Expression::Max(..)
            | Expression::Min(..)
            | Expression::Mod(..)
            | Expression::And(..)
            | Expression::Or(..)
            | Expression::Equals(..)
            | Expression::GreaterThen(..)
            | Expression::LessThen(..)
            | Expression::If(..) => {}
        };
        show_value(value, ui);
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Expression::FilterStatusUnits(_, e)
            | Expression::FilterNoStatusUnits(_, e)
            | Expression::StatusEntity(_, e)
            | Expression::Dbg(e)
            | Expression::Ctx(e)
            | Expression::ToI(e)
            | Expression::Vec2E(e)
            | Expression::UnitVec(e)
            | Expression::VX(e)
            | Expression::VY(e)
            | Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sqr(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::Floor(e)
            | Expression::Ceil(e)
            | Expression::Fract(e)
            | Expression::SlotUnit(e)
            | Expression::RandomF(e)
            | Expression::RandomUnit(e)
            | Expression::ListCount(e) => [e].into(),
            Expression::MaxUnit(a, b)
            | Expression::RandomUnitSubset(a, b)
            | Expression::Vec2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::WithVar(_, a, b)
            | Expression::LessThen(a, b) => [a, b].into(),
            Expression::If(a, b, c) => [a, b, c].into(),
            Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits
            | Expression::Value(_)
            | Expression::Context(_)
            | Expression::OwnerState(_)
            | Expression::TargetState(_)
            | Expression::CasterState(_)
            | Expression::StatusState(_, _)
            | Expression::OwnerStateLast(_)
            | Expression::TargetStateLast(_)
            | Expression::CasterStateLast(_)
            | Expression::StatusStateLast(_, _)
            | Expression::AbilityContext(_, _)
            | Expression::AbilityState(_, _)
            | Expression::StatusCharges(_)
            | Expression::HexColor(_)
            | Expression::F(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::S(_)
            | Expression::V2(_, _) => default(),
        }
    }
}

impl ShowEditor for Effect {
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Effect::Repeat(count, ef) => {
                count.show_node("count", context, world, ui);
                ef.show_node("effect", context, world, ui);
            }
            Effect::WithTarget(ex, ef) | Effect::WithOwner(ex, ef) | Effect::WithVar(_, ex, ef) => {
                ex.show_node("value", context, world, ui);
                ef.show_node("effect", context, world, ui);
            }
            Effect::AbilityStateAddVar(_, _, e) | Effect::Text(e) => {
                e.show_node("", context, world, ui)
            }

            Effect::Summon(_, e) => {
                if let Some(e) = e {
                    e.show_node("", context, world, ui);
                }
            }

            Effect::List(l) => {
                for e in l {
                    e.show_node("", context, world, ui);
                }
            }

            Effect::If(e, th, el) => {
                e.show_node("condition", context, world, ui);
                th.show_node("then", context, world, ui);
                el.show_node("else", context, world, ui);
            }
            Effect::StatusSetVar(target, _, _, value) | Effect::StateAddVar(_, target, value) => {
                target.show_node("target", context, world, ui);
                value.show_node("value", context, world, ui);
            }
            Effect::Noop
            | Effect::Damage
            | Effect::Kill
            | Effect::Heal
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses
            | Effect::FullCopy
            | Effect::ChangeStatus(_)
            | Effect::ClearStatus(_)
            | Effect::StealStatus(_)
            | Effect::UseAbility(_, _)
            | Effect::Vfx(_) => {}
        }
    }
    fn show_content(&mut self, _: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Effect::ChangeStatus(status)
            | Effect::ClearStatus(status)
            | Effect::StealStatus(status) => {
                status_selector(status, world, ui);
            }
            Effect::UseAbility(ability, base) => {
                ability_selector(ability, world, ui);
                DragValue::new(base).range(0..=10).ui(ui);
            }
            Effect::AbilityStateAddVar(ability, var, _) => {
                ability_selector(ability, world, ui);
                var_selector(var, ui);
            }
            Effect::StatusSetVar(_, status, var, _) => {
                status_selector(status, world, ui);
                var_selector(var, ui);
            }
            Effect::Summon(summon, _) => summon_selector(summon, world, ui),
            Effect::StateAddVar(var, _, _) | Effect::WithVar(var, _, _) => var_selector(var, ui),
            Effect::Vfx(vfx) => vfx_selector(vfx, world, ui),
            Effect::List(l) => {
                if Button::click("+".into()).ui(ui).clicked() {
                    l.push(default());
                }
            }

            Effect::WithTarget(_, _)
            | Effect::WithOwner(_, _)
            | Effect::Repeat(_, _)
            | Effect::If(_, _, _)
            | Effect::Text(_)
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses
            | Effect::FullCopy
            | Effect::Noop
            | Effect::Damage
            | Effect::Kill
            | Effect::Heal => {}
        }
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Effect::WithTarget(_, e)
            | Effect::WithOwner(_, e)
            | Effect::WithVar(_, _, e)
            | Effect::Repeat(_, e) => [e].into(),
            Effect::List(l) => l.iter_mut().collect_vec(),
            Effect::If(_, a, b) => [a, b].into(),

            Effect::Summon(_, _)
            | Effect::Noop
            | Effect::Damage
            | Effect::Kill
            | Effect::Heal
            | Effect::ChangeStatus(_)
            | Effect::ClearStatus(_)
            | Effect::StealStatus(_)
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses
            | Effect::UseAbility(_, _)
            | Effect::AbilityStateAddVar(_, _, _)
            | Effect::Vfx(_)
            | Effect::StateAddVar(_, _, _)
            | Effect::StatusSetVar(_, _, _, _)
            | Effect::Text(_)
            | Effect::FullCopy => default(),
        }
    }
}

impl ShowEditor for FireTrigger {
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            FireTrigger::List(l) => {
                for t in l {
                    t.show_node("", context, world, ui);
                }
            }
            FireTrigger::Period(_, _, t) | FireTrigger::OnceAfter(_, t) => {
                t.show_node("", context, world, ui)
            }
            FireTrigger::If(e, t) => {
                e.show_node("condition", context, world, ui);
                t.show_node("", context, world, ui);
            }
            FireTrigger::None
            | FireTrigger::UnitUsedAbility(..)
            | FireTrigger::AllyUsedAbility(..)
            | FireTrigger::EnemyUsedAbility(..)
            | FireTrigger::AfterIncomingDamage
            | FireTrigger::AfterDamageTaken
            | FireTrigger::AfterDamageDealt
            | FireTrigger::BattleStart
            | FireTrigger::TurnStart
            | FireTrigger::TurnEnd
            | FireTrigger::BeforeStrike
            | FireTrigger::AfterStrike
            | FireTrigger::AllyDeath
            | FireTrigger::AnyDeath
            | FireTrigger::AllySummon
            | FireTrigger::EnemySummon
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => {}
        }
    }

    fn show_content(&mut self, _: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            FireTrigger::Period(_, delay, _) => {
                DragValue::new(delay).ui(ui);
            }
            FireTrigger::OnceAfter(delay, _) => {
                DragValue::new(delay).ui(ui);
            }
            FireTrigger::UnitUsedAbility(ability)
            | FireTrigger::AllyUsedAbility(ability)
            | FireTrigger::EnemyUsedAbility(ability) => ability_selector(ability, world, ui),
            FireTrigger::If(..)
            | FireTrigger::None
            | FireTrigger::List(_)
            | FireTrigger::AfterIncomingDamage
            | FireTrigger::AfterDamageTaken
            | FireTrigger::AfterDamageDealt
            | FireTrigger::BattleStart
            | FireTrigger::TurnStart
            | FireTrigger::TurnEnd
            | FireTrigger::BeforeStrike
            | FireTrigger::AfterStrike
            | FireTrigger::AllyDeath
            | FireTrigger::AnyDeath
            | FireTrigger::AllySummon
            | FireTrigger::EnemySummon
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => {}
        }
    }

    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            FireTrigger::List(l) => l.iter_mut().collect_vec(),
            FireTrigger::Period(_, _, t) | FireTrigger::If(_, t) | FireTrigger::OnceAfter(_, t) => {
                [t].into()
            }
            FireTrigger::None
            | FireTrigger::UnitUsedAbility(_)
            | FireTrigger::AllyUsedAbility(_)
            | FireTrigger::EnemyUsedAbility(_)
            | FireTrigger::AfterIncomingDamage
            | FireTrigger::AfterDamageTaken
            | FireTrigger::AfterDamageDealt
            | FireTrigger::BattleStart
            | FireTrigger::TurnStart
            | FireTrigger::TurnEnd
            | FireTrigger::BeforeStrike
            | FireTrigger::AfterStrike
            | FireTrigger::AllyDeath
            | FireTrigger::AnyDeath
            | FireTrigger::AllySummon
            | FireTrigger::EnemySummon
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => default(),
        }
    }
}

fn status_selector(status: &mut String, world: &World, ui: &mut Ui) {
    Selector::new("status").ui_iter(status, GameAssets::get(world).statuses.keys(), ui);
}
fn ability_selector(ability: &mut String, world: &World, ui: &mut Ui) {
    Selector::new("ability").ui_iter(ability, GameAssets::get(world).abilities.keys(), ui);
}
fn summon_selector(summon: &mut String, world: &World, ui: &mut Ui) {
    Selector::new("summon").ui_iter(summon, GameAssets::get(world).summons.keys(), ui);
}
fn vfx_selector(vfx: &mut String, world: &World, ui: &mut Ui) {
    Selector::new("vfx").ui_iter(vfx, GameAssets::get(world).vfxs.keys(), ui);
}
fn var_selector(var: &mut VarName, ui: &mut Ui) {
    Selector::new("var").ui_enum(var, ui);
}
fn show_value(value: Result<VarValue>, ui: &mut Ui) {
    let w = ui.available_width();
    ui.set_max_width(ui.min_size().x);
    match value {
        Ok(v) => v
            .cstr()
            .style(CstrStyle::Small)
            .as_label(ui)
            .truncate()
            .ui(ui),
        Err(e) => e
            .to_string()
            .cstr_cs(RED, CstrStyle::Small)
            .as_label(ui)
            .truncate()
            .ui(ui),
    };
    ui.set_max_width(w);
}
