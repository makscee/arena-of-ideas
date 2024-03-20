use super::*;

use bevy_egui::egui::ComboBox;
use convert_case::Casing;
use event::Event;
use strum_macros::Display;

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter)]
#[serde(deny_unknown_fields)]
pub enum Trigger {
    Fire {
        #[serde(default)]
        triggers: Vec<(FireTrigger, Option<String>)>,
        #[serde(default)]
        targets: Vec<(Expression, Option<String>)>,
        #[serde(default)]
        effects: Vec<(Effect, Option<String>)>,
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, EnumIter, Default, AsRefStr)]
pub enum FireTrigger {
    #[default]
    Noop,
    List(Vec<Box<FireTrigger>>),
    Period(usize, usize, Box<FireTrigger>),
    OnceAfter(i32, Box<FireTrigger>),
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
    fn catch(&mut self, event: &Event, context: &Context, world: &World) -> bool {
        match self {
            FireTrigger::List(list) => list.iter_mut().any(|t| t.catch(event, context, world)),
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
            FireTrigger::Period(counter, delay, trigger) => {
                if !trigger.catch(event, context, world) {
                    return false;
                }
                if *counter == *delay {
                    *counter = 0;
                    true
                } else {
                    *counter += 1;
                    false
                }
            }
            FireTrigger::OnceAfter(counter, trigger) => {
                if !trigger.catch(event, context, world) {
                    return false;
                }
                *counter -= 1;
                *counter == -1
            }
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
            FireTrigger::Period(_, delay, t) => {
                format!("{} ({})", t.get_description_string(), *delay + 1)
            }
            _ => self.to_string(),
        }
    }
}

impl EditorNodeGenerator for FireTrigger {
    fn node_color(&self) -> Color32 {
        match self {
            FireTrigger::Noop
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
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => hex_color!("#80D8FF"),
            FireTrigger::Period(..) | FireTrigger::OnceAfter(..) => hex_color!("#18FFFF"),
            FireTrigger::List(_) => hex_color!("#FFEB3B"),
        }
    }

    fn show_children(
        &mut self,
        path: &str,
        connect_pos: Option<Pos2>,
        context: &Context,
        ui: &mut Ui,
        world: &mut World,
    ) {
        match self {
            FireTrigger::List(list) => {
                ui.vertical(|ui| {
                    for (i, eff) in list.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            show_node(
                                eff.as_mut(),
                                format!("{path}:t{i}"),
                                connect_pos,
                                context,
                                ui,
                                world,
                            );
                        });
                    }
                    if ui.button("+").clicked() {
                        list.push(default());
                    }
                });
            }
            FireTrigger::Period(_, _, t) | FireTrigger::OnceAfter(_, t) => show_node(
                t.as_mut(),
                format!("{path}/t"),
                connect_pos,
                context,
                ui,
                world,
            ),
            FireTrigger::Noop
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
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => default(),
        }
    }

    fn show_extra(&mut self, _: &str, _: &Context, _: &mut World, ui: &mut Ui) {
        match self {
            FireTrigger::List(list) => {
                if ui.button("CLEAR").clicked() {
                    list.clear()
                }
            }
            FireTrigger::Period(_, delay, _) => {
                DragValue::new(delay).ui(ui);
            }
            FireTrigger::OnceAfter(delay, _) => {
                DragValue::new(delay).ui(ui);
            }
            FireTrigger::Noop
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
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => {}
        }
    }

    fn show_replace_buttons(&mut self, lookup: &str, submit: bool, ui: &mut Ui) -> bool {
        for e in FireTrigger::iter() {
            if e.as_ref().to_lowercase().contains(lookup) {
                let btn = e.as_ref().add_color(e.node_color()).rich_text(ui);
                let btn = ui.button(btn);
                if btn.clicked() || submit {
                    btn.request_focus();
                }
                if btn.gained_focus() {
                    *self = e;
                    return true;
                }
            }
        }
        false
    }

    fn wrap(&mut self) {
        *self = FireTrigger::List([Box::new(self.clone())].into());
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
            triggers: default(),
            targets: default(),
            effects: default(),
        }
    }
}

impl Trigger {
    pub fn fire(&mut self, event: &Event, context: &Context, world: &mut World) -> bool {
        match self {
            Trigger::List(list) => {
                let mut result = false;
                for trigger in list {
                    result |= trigger.fire(event, context, world);
                }
                result
            }
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                if !triggers
                    .into_iter()
                    .any(|(trigger, _)| trigger.catch(event, context, world))
                {
                    return false;
                }
                for (effect, _) in effects {
                    if targets.is_empty() {
                        ActionPlugin::action_push_back(effect.clone(), context.clone(), world);
                    } else {
                        for (target, _) in targets.iter() {
                            let effect =
                                Effect::WithTarget(target.clone(), Box::new(effect.clone()));
                            ActionPlugin::action_push_back(effect, context.clone(), world);
                        }
                    }
                }
                true
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
                triggers,
                targets,
                effects,
            } => {
                let trigger = triggers
                    .into_iter()
                    .map(|(t, s)| s.clone().unwrap_or_else(|| t.to_string()))
                    .join(" & ");
                let effect = effects
                    .into_iter()
                    .map(|(e, s)| s.clone().unwrap_or_else(|| e.to_string()))
                    .join(" & ");
                let target = targets
                    .into_iter()
                    .map(|(t, s)| s.clone().unwrap_or_else(|| t.to_string()))
                    .join(" & ");
                state
                    .init(VarName::TriggerDescription, VarValue::String(trigger))
                    .init(VarName::EffectDescription, VarValue::String(effect))
                    .init(VarName::TargetDescription, VarValue::String(target));
            }
            Trigger::Change { .. } => {}
            Trigger::List(list) => list.iter().for_each(|t| t.inject_description(state)),
        }
    }
}

impl std::fmt::Display for FireTrigger {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        match self {
            FireTrigger::Noop
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
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => {
                write!(f, "{}", self.as_ref().to_case(convert_case::Case::Lower))
            }
            FireTrigger::List(list) => write!(
                f,
                "({})",
                list.into_iter().map(|t| t.to_string()).join(" + ")
            ),
            FireTrigger::Period(_, delay, trigger) => {
                write!(f, "
                {} ({delay} {trigger})", self.as_ref())
            }
            FireTrigger::OnceAfter(delay, trigger) => {
                write!(f, "{} ({delay} {trigger})", self.as_ref())
            }
        }
    }
}
