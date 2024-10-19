use super::*;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, EnumIter, Default, AsRefStr)]
pub enum FireTrigger {
    #[default]
    None,
    List(Vec<Box<FireTrigger>>),
    Period(usize, usize, Box<FireTrigger>),
    OnceAfter(i32, Box<FireTrigger>),
    UnitUsedAbility(String),
    AllyUsedAbility(String),
    EnemyUsedAbility(String),
    If(Expression, Box<FireTrigger>),
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
    pub fn catch(&mut self, event: &Event, context: &Context, world: &mut World) -> bool {
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
            FireTrigger::If(cond, trigger) => {
                cond.get_bool(context, world).unwrap_or_default()
                    && trigger.catch(event, context, world)
            }
            FireTrigger::None => false,
        }
    }
}
impl ShowEditor for FireTrigger {
    fn wrapper() -> Option<Self> {
        Some(Self::List([default()].into()))
    }
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
            | FireTrigger::EnemyUsedAbility(ability) => {
                ability_selector(ability, ui);
            }
            FireTrigger::List(l) => {
                if Button::click("+").ui(ui).clicked() {
                    l.push(default());
                }
            }
            FireTrigger::If(..)
            | FireTrigger::None
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
    fn get_variants() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

impl ToCstr for FireTrigger {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(match self {
            FireTrigger::None => visible_light(),
            FireTrigger::List(_) | FireTrigger::Period(_, _, _) | FireTrigger::OnceAfter(_, _) => {
                RED
            }
            FireTrigger::UnitUsedAbility(_)
            | FireTrigger::AllyUsedAbility(_)
            | FireTrigger::EnemyUsedAbility(_) => PURPLE,
            FireTrigger::If(_, _) => CYAN,

            FireTrigger::AfterIncomingDamage
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
            | FireTrigger::AfterKill => YELLOW,
        })
    }
    fn cstr_expanded(&self) -> Cstr {
        match self {
            FireTrigger::List(list) => {
                Cstr::join_vec(list.iter().map(|t| t.cstr_c(visible_light())).collect_vec())
                    .join(&" + ".cstr_c(visible_dark()))
                    .take()
            }
            FireTrigger::Period(_, delay, trigger) => format!("Every {delay} ")
                .cstr_c(visible_light())
                .push(trigger.cstr_expanded())
                .take(),
            FireTrigger::OnceAfter(delay, trigger) => format!("Once in {delay} ")
                .cstr_c(visible_light())
                .push(trigger.cstr_expanded())
                .take(),
            FireTrigger::If(cond, trigger) => trigger
                .cstr_expanded()
                .push(" if ".cstr())
                .push(cond.cstr_expanded())
                .take(),
            FireTrigger::UnitUsedAbility(name)
            | FireTrigger::AllyUsedAbility(name)
            | FireTrigger::EnemyUsedAbility(name) => self
                .as_ref()
                .to_case(Case::Lower)
                .cstr_c(visible_light())
                .push(format!(" {name}").cstr_cs(name_color(name), CstrStyle::Bold))
                .take(),
            FireTrigger::None
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
            | FireTrigger::AfterKill => self.as_ref().to_case(Case::Lower).cstr_c(visible_light()),
        }
    }
}
