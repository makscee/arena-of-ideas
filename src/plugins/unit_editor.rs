use egui::{DragValue, ScrollArea};

use super::*;

pub struct UnitEditorPlugin;

impl Plugin for UnitEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::UnitEditor), Self::on_enter)
            .init_resource::<UnitEditorResource>();
    }
}

#[derive(Resource, Default)]
struct UnitEditorResource {
    teams: HashMap<Faction, PackedTeam>,
    editing_hero: PackedUnit,
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
        for faction in [Faction::Left, Faction::Right] {
            if r.teams.contains_key(&faction) {
                continue;
            }
            r.teams.insert(faction, default());
        }
        Self::respawn_teams(world);
    }
    fn respawn_teams(world: &mut World) {
        TeamPlugin::despawn(Faction::Left, world);
        TeamPlugin::despawn(Faction::Right, world);
        for (faction, team) in rm(world).teams.clone() {
            team.clone().unpack(faction, world);
        }
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
    fn load_unit_editor(faction: Faction, slot: usize, world: &mut World) {
        let mut r = rm(world);
        r.editing_faction = faction;
        r.editing_slot = slot;
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
            let trigger = hero.trigger.to_string();
            ui.horizontal(|ui| {
                "trigger:".cstr().label(ui);
                if Button::click(trigger).ui(ui).clicked() {
                    let mut r = rm(world);
                    r.editing_trigger = r.editing_hero.trigger.clone();
                    Confirmation::new("Trigger Editor".cstr(), |world| {
                        let mut r = rm(world);
                        r.editing_hero.trigger = mem::take(&mut r.editing_trigger);
                    })
                    .content(|ui, world| {
                        rm(world).editing_trigger.show_editor(ui);
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
                        if entity.is_none() {
                            Self::open_new_unit_picker(faction, slot, ctx, world);
                        } else {
                            Self::load_unit_editor(faction, slot, world);
                            Self::open_unit_editor(ctx);
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

pub trait ShowEditor {
    fn show_editor(&mut self, ui: &mut Ui);
}

impl ShowEditor for Trigger {
    fn show_editor(&mut self, ui: &mut Ui) {
        match self {
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                for (target, name) in targets {
                    target.show_editor(ui);
                }
            }
            Trigger::Change { trigger, expr } => todo!(),
            Trigger::List(_) => todo!(),
        }
    }
}

impl ShowEditor for Expression {
    fn show_editor(&mut self, ui: &mut Ui) {
        ui.menu_button(self.as_ref().to_string(), |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                for e in Self::iter() {
                    if Button::click(e.as_ref().into()).ui(ui).clicked() {
                        *self = e;
                        ui.close_menu();
                    }
                }
            });
        });
        return;
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
            | Expression::AdjacentUnits => {}
            Expression::FilterStatusUnits(text, e)
            | Expression::FilterNoStatusUnits(text, e)
            | Expression::StatusEntity(text, e) => {
                Input::new("s:").ui_string(text, ui);
                e.show_editor(ui);
            }
            Expression::Value(_) => todo!(),
            Expression::Context(_) => todo!(),
            Expression::OwnerState(_) => todo!(),
            Expression::TargetState(_) => todo!(),
            Expression::CasterState(_) => todo!(),
            Expression::StatusState(_, _) => todo!(),
            Expression::OwnerStateLast(_) => todo!(),
            Expression::TargetStateLast(_) => todo!(),
            Expression::CasterStateLast(_) => todo!(),
            Expression::StatusStateLast(_, _) => todo!(),
            Expression::AbilityContext(_, _) => todo!(),
            Expression::AbilityState(_, _) => todo!(),
            Expression::StatusCharges(_) => todo!(),
            Expression::HexColor(_) => todo!(),
            Expression::F(_) => todo!(),
            Expression::I(_) => todo!(),
            Expression::B(_) => todo!(),
            Expression::S(_) => todo!(),
            Expression::V2(_, _) => todo!(),
            Expression::Dbg(_) => todo!(),
            Expression::Ctx(_) => todo!(),
            Expression::ToI(_) => todo!(),
            Expression::Vec2E(_) => todo!(),
            Expression::UnitVec(_) => todo!(),
            Expression::VX(_) => todo!(),
            Expression::VY(_) => todo!(),
            Expression::Sin(_) => todo!(),
            Expression::Cos(_) => todo!(),
            Expression::Sqr(_) => todo!(),
            Expression::Even(_) => todo!(),
            Expression::Abs(_) => todo!(),
            Expression::Floor(_) => todo!(),
            Expression::Ceil(_) => todo!(),
            Expression::Fract(_) => todo!(),
            Expression::SlotUnit(_) => todo!(),
            Expression::RandomF(_) => todo!(),
            Expression::RandomUnit(_) => todo!(),
            Expression::ListCount(_) => todo!(),
            Expression::MaxUnit(_, _) => todo!(),
            Expression::RandomUnitSubset(_, _) => todo!(),
            Expression::Vec2EE(_, _) => todo!(),
            Expression::Sum(_, _) => todo!(),
            Expression::Sub(_, _) => todo!(),
            Expression::Mul(_, _) => todo!(),
            Expression::Div(_, _) => todo!(),
            Expression::Max(_, _) => todo!(),
            Expression::Min(_, _) => todo!(),
            Expression::Mod(_, _) => todo!(),
            Expression::And(_, _) => todo!(),
            Expression::Or(_, _) => todo!(),
            Expression::Equals(_, _) => todo!(),
            Expression::GreaterThen(_, _) => todo!(),
            Expression::LessThen(_, _) => todo!(),
            Expression::If(_, _, _) => todo!(),
            Expression::WithVar(_, _, _) => todo!(),
        }
    }
}
