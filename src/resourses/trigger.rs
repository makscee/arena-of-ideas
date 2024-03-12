use super::*;

use bevy_egui::egui::ComboBox;
use event::Event;
use strum_macros::Display;

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter)]
pub enum Trigger {
    Fire {
        trigger: FireTrigger,
        #[serde(default = "owner")]
        target: Expression,
        effect: Effect,
        #[serde(default)]
        period: usize,
    },
    Change {
        trigger: DeltaTrigger,
        expr: Expression,
    },
    List(Vec<Box<Trigger>>),
}

fn owner() -> Expression {
    Expression::Owner
}

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter, Default)]
pub enum DeltaTrigger {
    #[default]
    IncomingDamage,
    Var(VarName),
}

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter, Default)]
pub enum FireTrigger {
    #[default]
    Noop,
    List(Vec<Box<FireTrigger>>),
    AfterIncomingDamage,
    AfterDamageTaken,
    AfterDamageDealt,
    BattleStart,
    TurnStart,
    TurnEnd,
    BeforeStrike,
    AfterStrike,
    AllyDeath,
    AnyDeath,
    AllySummon,
    BeforeDeath,
    AfterKill,
}

impl FireTrigger {
    fn catch(&self, event: &Event, context: &Context, world: &mut World) -> bool {
        match self {
            FireTrigger::List(list) => list.iter().any(|t| t.catch(event, context, world)),
            FireTrigger::AfterIncomingDamage => matches!(event, Event::IncomingDamage { .. }),
            FireTrigger::AfterDamageTaken => matches!(event, Event::DamageTaken { .. }),
            FireTrigger::AfterDamageDealt => matches!(event, Event::DamageDealt { .. }),
            FireTrigger::BattleStart => matches!(event, Event::BattleStart { .. }),
            FireTrigger::TurnStart => matches!(event, Event::TurnStart { .. }),
            FireTrigger::TurnEnd => matches!(event, Event::TurnEnd { .. }),
            FireTrigger::BeforeStrike => matches!(event, Event::BeforeStrike { .. }),
            FireTrigger::AfterStrike => matches!(event, Event::AfterStrike { .. }),
            FireTrigger::AfterKill => matches!(event, Event::Kill { .. }),
            FireTrigger::AnyDeath => matches!(event, Event::Death { .. }),
            FireTrigger::AllyDeath => match event {
                Event::Death(dead) => UnitPlugin::get_faction(*dead, world)
                    .eq(&UnitPlugin::get_faction(context.owner(), world)),
                _ => false,
            },
            FireTrigger::AllySummon => match event {
                Event::Summon(e) => UnitPlugin::get_faction(*e, world)
                    .eq(&UnitPlugin::get_faction(context.owner(), world)),
                _ => false,
            },
            FireTrigger::BeforeDeath => match event {
                Event::Death(dead) => dead.eq(&context.owner()),
                _ => false,
            },
            FireTrigger::Noop => false,
        }
    }

    pub fn show_editor(&mut self, name: impl std::hash::Hash, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::from_id_source(name)
                .selected_text(self.to_string())
                .width(150.0)
                .show_ui(ui, |ui| {
                    for option in FireTrigger::iter() {
                        let text = option.to_string();
                        ui.selectable_value(self, option, text);
                    }
                });
        });
    }

    fn get_description_string(&self) -> String {
        match self {
            FireTrigger::List(list) => list.iter().map(|t| t.get_description_string()).join(" + "),
            _ => self.to_string(),
        }
    }
}

impl DeltaTrigger {
    fn catch(&self, event: &Event) -> bool {
        match self {
            DeltaTrigger::IncomingDamage => matches!(event, Event::IncomingDamage { .. }),
            DeltaTrigger::Var(..) => false,
        }
    }
}

impl Default for Trigger {
    fn default() -> Self {
        Self::Fire {
            trigger: FireTrigger::Noop,
            target: Expression::Owner,
            effect: Effect::Noop,
            period: 0,
        }
    }
}

impl Trigger {
    pub fn fire(&self, event: &Event, context: &Context, world: &mut World) -> bool {
        match self {
            Trigger::List(list) => {
                let mut result = false;
                for trigger in list {
                    result |= trigger.fire(event, context, world);
                }
                result
            }
            Trigger::Fire {
                trigger,
                target,
                effect,
                period,
            } => {
                if !trigger.catch(event, context, world) {
                    return false;
                }
                let mut state = VarState::get_mut(context.status(), world);
                let count = state.get_int(VarName::Count).unwrap_or_default() + 1;
                if count > *period as i32 {
                    state.set_int(VarName::Count, 0);
                } else {
                    state.set_int(VarName::Count, count + 1);
                    return false;
                }
                let effect = Effect::WithTarget(target.clone(), Box::new(effect.clone()));
                match trigger {
                    FireTrigger::List(_)
                    | FireTrigger::AfterDamageTaken
                    | FireTrigger::AfterDamageDealt
                    | FireTrigger::BattleStart
                    | FireTrigger::BeforeStrike
                    | FireTrigger::AfterStrike
                    | FireTrigger::AllyDeath
                    | FireTrigger::AnyDeath
                    | FireTrigger::AllySummon
                    | FireTrigger::BeforeDeath
                    | FireTrigger::AfterKill
                    | FireTrigger::AfterIncomingDamage
                    | FireTrigger::TurnStart
                    | FireTrigger::TurnEnd => {
                        ActionPlugin::action_push_back(effect, context.clone(), world);
                        true
                    }

                    FireTrigger::Noop => false,
                }
            }
            Trigger::Change { .. } => false,
        }
    }

    pub fn change(
        &self,
        event: &Event,
        context: &Context,
        value: &mut VarValue,
        world: &mut World,
    ) -> Result<()> {
        match self {
            Trigger::List(list) => list.iter().for_each(|t| {
                let _ = t.change(event, context, value, world);
            }),
            Trigger::Change { trigger, expr } => {
                if !trigger.catch(event) {
                    return Ok(());
                }
                let delta = expr.get_value(
                    context.clone().set_var(VarName::Value, value.clone()),
                    world,
                )?;
                *value = VarValue::sum(value, &delta)?;
            }
            Trigger::Fire { .. } => {}
        };
        Ok(())
    }

    pub fn collect_mappings(
        &self,
        context: &Context,
        world: &mut World,
    ) -> Vec<(VarName, VarValue)> {
        match self {
            Trigger::List(list) => list
                .iter()
                .flat_map(|t| t.collect_mappings(context, world))
                .collect_vec(),
            Trigger::Change { trigger, expr } => match trigger {
                DeltaTrigger::IncomingDamage => default(),
                DeltaTrigger::Var(var) => match expr.get_value(context, world) {
                    Ok(value) => [(*var, value)].into(),
                    Err(_) => default(),
                },
            },
            Trigger::Fire { .. } => default(),
        }
    }

    pub fn has_stat_change(&self) -> bool {
        match self {
            Trigger::List(list) => list.iter().any(|t| t.has_stat_change()),
            Trigger::Change { .. } => true,
            Trigger::Fire { .. } => false,
        }
    }

    pub fn inject_description(&self, state: &mut VarState) {
        match self {
            Trigger::Fire {
                trigger,
                target,
                effect,
                period,
            } => {
                let mut trigger = trigger.get_description_string();
                if *period > 0 {
                    let s = format!(" ({})", *period + 1);
                    trigger.push_str(&s);
                }
                state
                    .init(VarName::TriggerDescription, VarValue::String(trigger))
                    .init(
                        VarName::EffectDescription,
                        VarValue::String(
                            effect
                                .find_all_abilities()
                                .into_iter()
                                .map(|a| match a {
                                    Effect::UseAbility(ability, mult) => {
                                        format!(
                                            "[{ability}] ({{Level}}{})",
                                            if mult > 1 {
                                                format!("x{mult}")
                                            } else {
                                                default()
                                            }
                                        )
                                    }
                                    _ => default(),
                                })
                                .join(" + "),
                        ),
                    )
                    .init(
                        VarName::TargetDescription,
                        VarValue::String(target.get_description_string()),
                    );
            }
            Trigger::Change { .. } => {}
            Trigger::List(list) => list.iter().for_each(|t| t.inject_description(state)),
        }
    }
}
