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
            FireTrigger::AnyDeath => matches!(event, Event::Death { .. }),
            FireTrigger::AfterKill => matches!(event, Event::Kill { .. }),
            FireTrigger::AllyDeath => match event {
                Event::Death(dead) => UnitPlugin::get_faction(*dead, world)
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

    fn show_editor(&mut self, name: String, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::from_id_source(&name)
                .selected_text(self.to_string())
                .width(150.0)
                .show_ui(ui, |ui| {
                    for option in FireTrigger::iter() {
                        let text = option.to_string();
                        ui.selectable_value(self, option, text).changed();
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

    fn show_editor(&mut self, name: String, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::from_id_source(&name)
                .selected_text(self.to_string())
                .width(150.0)
                .show_ui(ui, |ui| {
                    for option in DeltaTrigger::iter() {
                        let text = option.to_string();
                        ui.selectable_value(self, option, text).changed();
                    }
                });
            match self {
                DeltaTrigger::Var(var) => var.show_editor(ui),
                DeltaTrigger::IncomingDamage => {}
            }
        });
    }
}

impl Default for Trigger {
    fn default() -> Self {
        Self::Fire {
            trigger: FireTrigger::Noop,
            target: Expression::Owner,
            effect: Effect::Noop,
        }
    }
}

impl Trigger {
    pub fn fire(&self, event: &Event, context: &Context, world: &mut World) {
        match self {
            Trigger::List(list) => list.iter().for_each(|t| t.fire(event, context, world)),
            Trigger::Fire {
                trigger,
                target,
                effect,
            } => {
                if !trigger.catch(event, context, world) {
                    return;
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
                    | FireTrigger::BeforeDeath
                    | FireTrigger::AfterKill
                    | FireTrigger::AfterIncomingDamage
                    | FireTrigger::TurnStart
                    | FireTrigger::TurnEnd => {
                        ActionCluster::current(world).push_action_back(effect, context.clone());
                    }

                    FireTrigger::Noop => {}
                }
            }
            Trigger::Change { .. } => {}
        };
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
                Trigger::Fire {
                    trigger,
                    target,
                    effect,
                } => {
                    ui.vertical(|ui| {
                        trigger.show_editor(format!("{name}/trigger"), ui);
                        target.show_editor(editing_data, format!("{name}/target"), ui);
                        effect.show_editor(editing_data, format!("{name}/effect"), ui, world);
                    });
                }
                Trigger::Change {
                    trigger,
                    expr: expression,
                } => {
                    ui.vertical(|ui| {
                        expression.show_editor(editing_data, format!("{name}/exp"), ui);
                        trigger.show_editor(format!("{name}/trigger"), ui);
                    });
                }
                Trigger::List(list) => {
                    ui.vertical(|ui| {
                        list.iter_mut().enumerate().for_each(|(i, t)| {
                            t.show_editor(editing_data, format!("{name} {i}"), ui, world);
                        });
                    });
                }
            };
        });
    }

    pub fn inject_description(&self, state: &mut VarState) {
        match self {
            Trigger::Fire {
                trigger,
                target,
                effect,
            } => {
                state
                    .init(
                        VarName::TriggerDescription,
                        VarValue::String(trigger.get_description_string()),
                    )
                    .init(
                        VarName::EffectDescription,
                        VarValue::String(
                            effect
                                .find_all_abilities()
                                .into_iter()
                                .map(|a| match a {
                                    Effect::UseAbility(ability) => {
                                        format!("[{ability}] ({{Level}})")
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
