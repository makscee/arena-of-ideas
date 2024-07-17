use super::*;

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
    UnitUsedAbility(String),
    AllyUsedAbility(String),
    EnemyUsedAbility(String),
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
    EnemySummon,
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
            FireTrigger::AnyDeath => {
                matches!(event, Event::Death (dead) if !context.owner().eq(dead))
            }
            FireTrigger::AllyDeath => match event {
                Event::Death(dead) => {
                    !context.owner().eq(dead)
                        && dead.faction(world).eq(&context.owner().faction(world))
                }
                _ => false,
            },
            FireTrigger::AllySummon => match event {
                Event::Summon(e) => e.faction(world).eq(&context.owner().faction(world)),
                _ => false,
            },
            FireTrigger::EnemySummon => match event {
                Event::Summon(e) => e
                    .faction(world)
                    .eq(&context.owner().faction(world).opposite()),
                _ => false,
            },
            FireTrigger::UnitUsedAbility(name) => match event {
                Event::UseAbility(e) => e.eq(name),
                _ => false,
            },
            FireTrigger::AllyUsedAbility(name) => match event {
                Event::UseAbility(e) => {
                    e.eq(name)
                        && context
                            .owner()
                            .faction(world)
                            .eq(&context.caster().faction(world))
                }
                _ => false,
            },
            FireTrigger::EnemyUsedAbility(name) => match event {
                Event::UseAbility(e) => {
                    e.eq(name)
                        && context
                            .owner()
                            .faction(world)
                            .opposite()
                            .eq(&context.target().faction(world))
                }
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
                let mut result = false;
                for (trigger, _) in triggers {
                    if trigger.catch(event, context, world) {
                        result = true;
                        for (effect, _) in effects.iter() {
                            match effect {
                                Effect::UseAbility(name, _) => {
                                    Event::UseAbility(name.clone()).send_with_context(
                                        context.clone().set_caster(context.owner()).take(),
                                        world,
                                    );
                                }
                                _ => {}
                            }
                            if targets.is_empty() {
                                ActionPlugin::action_push_back(
                                    effect.clone(),
                                    context.clone(),
                                    world,
                                );
                            } else {
                                for (target, _) in targets.iter() {
                                    let effect = Effect::WithTarget(
                                        target.clone(),
                                        Box::new(effect.clone()),
                                    );
                                    ActionPlugin::action_push_back(effect, context.clone(), world);
                                }
                            }
                        }
                    }
                }
                result
            }
            Trigger::Change { .. } => false,
        }
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
                    Err(e) => {
                        debug!("{} {e}", "Mapping error:".red());
                        default()
                    }
                },
            },
            Trigger::Fire { .. } => default(),
        }
    }
    pub fn parse_fire_strings(&self) -> (Vec<Cstr>, Vec<Cstr>, Vec<Cstr>) {
        let mut cs = (Vec::new(), Vec::new(), Vec::new());
        match self {
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => {
                for (trigger, rename) in triggers {
                    if let Some(rename) = rename {
                        cs.0.push(rename.cstr_c(VISIBLE_BRIGHT));
                    } else {
                        cs.0.push(trigger.cstr());
                    }
                }
                for (target, rename) in targets {
                    if let Some(rename) = rename {
                        cs.1.push(rename.cstr_c(VISIBLE_BRIGHT));
                    } else {
                        cs.1.push(target.cstr());
                    }
                }
                for (effect, rename) in effects {
                    if let Some(rename) = rename {
                        cs.2.push(Cstr::parse(rename).take());
                    } else {
                        cs.2.push(effect.cstr());
                    }
                }
            }
            _ => panic!("Has to be Trigger::Fire"),
        }
        cs
    }
}

impl ToCstr for FireTrigger {
    fn cstr(&self) -> Cstr {
        match self {
            FireTrigger::List(list) => {
                Cstr::join_vec(list.iter().map(|t| t.cstr_c(VISIBLE_BRIGHT)).collect_vec())
                    .join(&" + ".cstr_c(VISIBLE_DARK))
                    .take()
            }
            FireTrigger::Period(_, delay, trigger) => format!("Every {delay} ")
                .cstr()
                .push(trigger.cstr())
                .color(VISIBLE_BRIGHT)
                .take(),
            FireTrigger::OnceAfter(delay, trigger) => format!("Once in {delay} ")
                .cstr()
                .push(trigger.cstr())
                .color(VISIBLE_BRIGHT)
                .take(),
            FireTrigger::UnitUsedAbility(name)
            | FireTrigger::AllyUsedAbility(name)
            | FireTrigger::EnemyUsedAbility(name) => self
                .as_ref()
                .to_case(Case::Lower)
                .cstr_c(VISIBLE_BRIGHT)
                .push(format!(" {name}").cstr_cs(name_color(name), CstrStyle::Bold))
                .take(),
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
            | FireTrigger::EnemySummon
            | FireTrigger::BeforeDeath
            | FireTrigger::AfterKill => self.as_ref().to_case(Case::Lower).cstr_c(VISIBLE_BRIGHT),
        }
    }
}
