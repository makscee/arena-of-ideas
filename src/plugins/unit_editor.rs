use egui::{Checkbox, DragValue, Key, ScrollArea};

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
        Self::respawn_teams(world);
    }
    fn on_exit(world: &mut World) {
        TeamPlugin::despawn(Faction::Left, world);
        TeamPlugin::despawn(Faction::Right, world);
    }
    fn respawn_teams(world: &mut World) {
        TeamPlugin::despawn(Faction::Left, world);
        TeamPlugin::despawn(Faction::Right, world);
        for (faction, team) in rm(world).teams.clone() {
            team.unpack(faction, world);
        }
        let mut cs = client_state().clone();
        cs.editor_teams = rm(world).teams.clone();
        cs.save();
    }
    fn set_team_unit(unit: PackedUnit, faction: Faction, slot: usize, world: &mut World) {
        let mut r = rm(world);
        let team = r.teams.get_mut(&faction).unwrap();
        if team.units.len() <= slot {
            team.units.push(unit);
        } else {
            team.units.remove(slot);
            team.units.insert(slot, unit);
        }
        UnitEditorPlugin::respawn_teams(world);
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
                Self::respawn_teams(world);
                Confirmation::pop(ui.ctx());
            }
            let mut r = rm(world);
            Selector::new("spawn").ui_vec(
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
                        trigger.show_node(&context, world, ui);
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

                    Self::respawn_teams(world);
                }
                if Button::click("Strike".into()).ui(ui).clicked() {
                    if let Some((left, right)) = BattlePlugin::get_strikers(world) {
                        BattlePlugin::run_strike(left, right, world);
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
            ui.columns(2, |ui| {
                TeamContainer::new(Faction::Left)
                    .right_to_left()
                    .on_click(|slot, entity, world| {
                        let ctx = &egui_context(world).unwrap();
                        if Confirmation::has_active(ctx) {
                            return;
                        }
                        let faction = Faction::Left;
                        if let Some(entity) = entity {
                            Self::load_unit_editor(entity, faction, slot, world);
                            Self::open_unit_editor(ctx);
                        } else {
                            Self::open_new_unit_picker(faction, slot, ctx, world);
                        }
                    })
                    .ui(&mut ui[0], world);
                TeamContainer::new(Faction::Right).ui(&mut ui[1], world);
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
pub trait ShowEditor: ToCstr + IntoEnumIterator {
    fn show_node(&mut self, context: &Context, world: &mut World, ui: &mut Ui);
    fn show_value(&mut self, _context: &Context, _world: &mut World, _ui: &mut Ui) {}
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui);
    fn show_self(&mut self, ui: &mut Ui) {
        let widget = self.cstr().widget(1.0, ui);
        if ui
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
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    ui.reset_style();
                    let lookup = lookup_text(ui.ctx()).to_lowercase();
                    for e in Self::iter() {
                        let c = e.cstr();
                        if !lookup.is_empty() && !c.get_text().to_lowercase().starts_with(&lookup) {
                            continue;
                        }
                        if c.button(ui).clicked() || take_first {
                            *self = e;
                            ui.close_menu();
                            break;
                        }
                    }
                });
            })
            .response
            .clicked()
        {
            lookup_text_clear(ui.ctx());
        }
    }
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
            node.show_node(context, world, ui);
        });
    });
}
impl ShowEditor for Trigger {
    fn show_node(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        self.show_self(ui);
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
                ui.collapsing("Targets", |ui| {
                    for (node, name) in targets {
                        show_named_node(name, node, context, world, ui);
                    }
                });
            }
            Trigger::Change { trigger, expr } => todo!(),
            Trigger::List(_) => todo!(),
        }
    }
}

impl ShowEditor for Expression {
    fn show_node(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
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
        FRAME.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.show_self(ui);
                    ui.set_max_width(ui.min_size().x);
                    self.show_value(context, world, ui);
                });
                self.show_children(context, world, ui);
            });
        });
    }

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
            | Expression::ListCount(e) => e.show_node(context, world, ui),

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
            | Expression::LessThen(a, b)
            | Expression::WithVar(_, a, b) => {
                ui.vertical(|ui| {
                    a.show_node(context, world, ui);
                    b.show_node(context, world, ui);
                });
            }
            Expression::If(a, b, c) => {
                ui.vertical(|ui| {
                    a.show_node(context, world, ui);
                    b.show_node(context, world, ui);
                    c.show_node(context, world, ui);
                });
            }
        }
    }

    fn show_value(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        let value = self.get_value(context, world);
        match self {
            Expression::FilterStatusUnits(text, e)
            | Expression::FilterNoStatusUnits(text, e)
            | Expression::StatusEntity(text, e) => {
                Input::new("s:").ui_string(text, ui);
            }
            Expression::Value(v) => todo!(),
            Expression::Context(var)
            | Expression::OwnerState(var)
            | Expression::TargetState(var)
            | Expression::CasterState(var)
            | Expression::OwnerStateLast(var)
            | Expression::TargetStateLast(var)
            | Expression::CasterStateLast(var) => {
                Selector::new("var").ui_enum(var, ui);
            }
            Expression::StatusState(status, var)
            | Expression::StatusStateLast(status, var)
            | Expression::AbilityContext(status, var)
            | Expression::AbilityState(status, var) => {
                Input::new("status:").ui_string(status, ui);
                Selector::new("var").ui_enum(var, ui);
            }
            Expression::StatusCharges(status) => Input::new("status:").ui_string(status, ui),
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
            | Expression::If(..)
            | Expression::WithVar(..) => {}
        };
        match value {
            Ok(v) => v
                .cstr_cs(VISIBLE_DARK, CstrStyle::Small)
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
    }
}
