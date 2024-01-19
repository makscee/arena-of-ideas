use super::*;

use bevy_egui::egui::ComboBox;
use event::Event;
use strum_macros::Display;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Display, PartialEq, EnumIter)]
pub enum Trigger {
    AfterIncomingDamage(Effect),
    AfterDamageTaken(Effect),
    AfterDamageDealt(Effect),
    BattleStart(Effect),
    TurnStart(Effect),
    TurnEnd(Effect),
    BeforeStrike(Effect),
    AfterStrike(Effect),
    AllyDeath(Effect),
    AnyDeath(Effect),
    BeforeDeath(Effect),
    AfterKill(Effect),
    DeltaVar(VarName, Expression),
    MapVar(VarName, Expression),
    List(Vec<Box<Trigger>>),
    #[default]
    Noop,
}

impl Trigger {
    pub fn catch_event(&self, event: &Event) -> Vec<Trigger> {
        match self {
            Trigger::Noop | Trigger::DeltaVar(..) | Trigger::MapVar(..) => default(),
            Trigger::List(triggers) => triggers
                .iter()
                .flat_map(|t| t.catch_event(event))
                .collect_vec(),
            Trigger::AfterIncomingDamage(..) => match event {
                Event::IncomingDamage { .. } => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterDamageTaken(..) => match event {
                Event::DamageTaken { .. } => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterDamageDealt(..) => match event {
                Event::DamageDealt { .. } => vec![self.clone()],
                _ => default(),
            },
            Trigger::BattleStart(..) => match event {
                Event::BattleStart => vec![self.clone()],
                _ => default(),
            },
            Trigger::TurnStart(..) => match event {
                Event::TurnStart => vec![self.clone()],
                _ => default(),
            },
            Trigger::TurnEnd(..) => match event {
                Event::TurnEnd => vec![self.clone()],
                _ => default(),
            },
            Trigger::BeforeStrike(..) => match event {
                Event::BeforeStrike(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterStrike(..) => match event {
                Event::AfterStrike(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::AllyDeath(..) | Trigger::AnyDeath(..) => match event {
                Event::Death(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::BeforeDeath(..) => match event {
                Event::Death(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterKill(..) => match event {
                Event::Kill { .. } => vec![self.clone()],
                _ => default(),
            },
        }
    }

    pub fn fire(self, event: &Event, context: &Context, status: Entity, world: &mut World) {
        let mut context = mem::take(
            context
                .clone()
                .set_owner(get_parent(status, world), world)
                .set_status(status, world),
        );
        match self {
            Trigger::AfterDamageTaken(effect)
            | Trigger::AfterDamageDealt(effect)
            | Trigger::BattleStart(effect)
            | Trigger::BeforeStrike(effect)
            | Trigger::AfterStrike(effect)
            | Trigger::AnyDeath(effect) => {
                ActionPlugin::new_cluster(effect, context, world);
            }
            Trigger::TurnStart(effect)
            | Trigger::TurnEnd(effect)
            | Trigger::AfterIncomingDamage(effect) => {
                ActionCluster::current(world).push_action_back(effect, context);
            }
            Trigger::AllyDeath(effect) => {
                let dead = match event {
                    Event::Death(unit) => *unit,
                    _ => panic!(),
                };
                let owner = get_parent(status, world);
                if UnitPlugin::get_faction(dead, world).eq(&UnitPlugin::get_faction(owner, world)) {
                    ActionPlugin::new_cluster(effect, context, world);
                }
            }
            Trigger::BeforeDeath(effect) => {
                let dead = match event {
                    Event::Death(unit) => *unit,
                    _ => panic!(),
                };
                let owner = get_parent(status, world);
                if dead.eq(&owner) {
                    context.stack(ContextLayer::DeadOwner, world);
                    ActionPlugin::new_cluster(effect, context, world);
                }
            }
            Trigger::AfterKill(effect) => {
                let target = match event {
                    Event::Kill { owner: _, target } => *target,
                    _ => panic!(),
                };
                context.set_target(target, world);
                ActionPlugin::new_cluster(effect, context, world);
            }
            Trigger::DeltaVar(_, _) | Trigger::MapVar(_, _) | Trigger::List(_) | Trigger::Noop => {
                panic!("Trigger {self} can not be fired")
            }
        }
    }

    pub fn collect_delta_triggers(&self) -> Vec<Trigger> {
        match self {
            Trigger::DeltaVar(_, _) => vec![self.clone()],
            Trigger::List(triggers) => triggers
                .iter()
                .flat_map(|t| t.collect_delta_triggers())
                .collect_vec(),
            _ => default(),
        }
    }

    pub fn collect_map_triggers(&self) -> Vec<Trigger> {
        match self {
            Trigger::MapVar(_, _) => vec![self.clone()],
            Trigger::List(triggers) => triggers
                .iter()
                .flat_map(|t| t.collect_map_triggers())
                .collect_vec(),
            _ => default(),
        }
    }

    pub fn show_editor(
        &mut self,
        editing_data: &mut EditingData,
        name: String,
        ui: &mut Ui,
        world: &mut World,
    ) {
        ui.horizontal(|ui| {
            ComboBox::from_id_source(&name)
                .selected_text(self.to_string())
                .width(150.0)
                .show_ui(ui, |ui| {
                    for option in Trigger::iter() {
                        let text = option.to_string();
                        ui.selectable_value(self, option, text).changed();
                    }
                });
            match self {
                Trigger::AfterDamageTaken(effect)
                | Trigger::AfterDamageDealt(effect)
                | Trigger::AfterIncomingDamage(effect)
                | Trigger::BattleStart(effect)
                | Trigger::TurnStart(effect)
                | Trigger::TurnEnd(effect)
                | Trigger::BeforeStrike(effect)
                | Trigger::AfterStrike(effect)
                | Trigger::AllyDeath(effect)
                | Trigger::AnyDeath(effect)
                | Trigger::BeforeDeath(effect)
                | Trigger::AfterKill(effect) => {
                    effect.show_editor(editing_data, format!("{name}/{effect}"), ui, world);
                }
                Trigger::DeltaVar(var, exp) | Trigger::MapVar(var, exp) => {
                    ui.vertical(|ui| {
                        var.show_editor(ui);
                        exp.show_editor(editing_data, format!("{name}/exp"), ui);
                    });
                }
                Trigger::List(list) => {
                    ui.vertical(|ui| {
                        list.iter_mut().enumerate().for_each(|(i, t)| {
                            t.show_editor(editing_data, format!("{name} {i}"), ui, world);
                        });
                    });
                }
                Trigger::Noop => {}
            }
        });
    }

    pub fn get_description_string(&self) -> String {
        let mut s = match self {
            Trigger::AfterIncomingDamage(_)
            | Trigger::AfterDamageTaken(_)
            | Trigger::AfterDamageDealt(_)
            | Trigger::BattleStart(_)
            | Trigger::TurnStart(_)
            | Trigger::TurnEnd(_)
            | Trigger::BeforeStrike(_)
            | Trigger::AfterStrike(_)
            | Trigger::AllyDeath(_)
            | Trigger::AnyDeath(_)
            | Trigger::BeforeDeath(_)
            | Trigger::DeltaVar(_, _)
            | Trigger::MapVar(_, _)
            | Trigger::Noop
            | Trigger::AfterKill(_) => self.to_string(),
            Trigger::List(triggers) => triggers.into_iter().map(|t| t.to_string()).join(" + "),
        };
        s.push_str(" → ");
        s
    }

    pub fn get_inner_effect(&mut self) -> Option<&mut Effect> {
        match self {
            Trigger::AfterIncomingDamage(effect)
            | Trigger::AfterDamageTaken(effect)
            | Trigger::AfterDamageDealt(effect)
            | Trigger::BattleStart(effect)
            | Trigger::TurnStart(effect)
            | Trigger::TurnEnd(effect)
            | Trigger::BeforeStrike(effect)
            | Trigger::AfterStrike(effect)
            | Trigger::AllyDeath(effect)
            | Trigger::AnyDeath(effect)
            | Trigger::BeforeDeath(effect)
            | Trigger::AfterKill(effect) => Some(effect),
            _ => None,
        }
    }

    pub fn set_inner_effect(&mut self, effect: Effect)  {
        if let Some(inner) = self.get_inner_effect() {
            *inner = effect;
        }
    }
}
