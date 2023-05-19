use geng::prelude::itertools::Itertools;
use strum_macros::AsRefStr;

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, AsRefStr)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Effect {
    Damage {
        value: Option<ExpressionInt>,
        on_hit: Option<Box<EffectWrapped>>,
    },
    Heal {
        value: Box<ExpressionInt>,
    },
    Kill,
    Repeat {
        count: ExpressionInt,
        effect: Box<EffectWrapped>,
    },
    List {
        items: Vec<Box<EffectWrapped>>,
    },
    Debug {
        message: String,
    },
    Noop,
    SetVarInt {
        var: VarName,
        value: ExpressionInt,
    },
    SetVarFaction {
        var: VarName,
        #[serde(default)]
        value: ExpressionFaction,
    },
    ChangeAbilityVarInt {
        ability: AbilityName,
        var: VarName,
        delta: ExpressionInt,
    },
    ChangeTeamVarInt {
        var: VarName,
        delta: ExpressionInt,
        faction: Option<ExpressionFaction>,
    },
    SetTeamVarInt {
        var: VarName,
        value: ExpressionInt,
        faction: Option<ExpressionFaction>,
    },
    ChangeOwnerVarInt {
        var: VarName,
        delta: ExpressionInt,
    },
    SetOwnerVarInt {
        var: VarName,
        value: ExpressionInt,
    },
    SetOwnerVarFaction {
        var: VarName,
        value: ExpressionFaction,
    },
    AddStatus {
        name: String,
    },
    AddTeamStatus {
        name: String,
    },
    RemoveStatus {
        name: String,
    },
    RemoveThisStatus,
    ClearStatuses,
    ChangeStatus {
        name: String,
        charges: ExpressionInt,
    },
    UseAbility {
        ability: AbilityName,
        #[serde(default)]
        force: bool,
        charges: Option<ExpressionInt>,
    },
    TakeVar {
        var: VarName,
        new_name: Option<VarName>,
        entity: ExpressionEntity,
        effect: Box<EffectWrapped>,
    },
    If {
        condition: Condition,
        then: Box<EffectWrapped>,
        r#else: Option<Box<EffectWrapped>>,
    },
    ShowText {
        text: String,
        color: Option<Rgba<f32>>,
        #[serde(default)]
        entity: Option<ExpressionEntity>,
        #[serde(default)]
        font: usize,
    },
    ShowCurve {
        color: Option<Rgba<f32>>,
    },
    Aoe {
        factions: Vec<ExpressionFaction>,
        effect: Box<EffectWrapped>,
        #[serde(default)]
        exclude_self: bool,
    },
    Revive {
        slot: Option<ExpressionInt>,
    },
    RemoveTrigger,
    /// Do effect if a unit matches condition
    FindTarget {
        faction: ExpressionFaction,
        condition: Condition,
        effect: Box<EffectWrapped>,
        #[serde(default)]
        exclude_self: bool,
    },
    AllTargets {
        faction: ExpressionFaction,
        condition: Condition,
        effect: Box<EffectWrapped>,
    },
    Summon {
        unit: Box<PackedUnit>,
        slot: Option<ExpressionInt>,
    },
}

impl Effect {
    pub fn wrap(self) -> EffectWrapped {
        EffectWrapped {
            effect: self,
            target: default(),
            owner: default(),
            after: default(),
            vars: default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EffectWrapped {
    #[serde(flatten)]
    pub effect: Effect,
    pub target: Option<ExpressionEntity>,
    pub owner: Option<ExpressionEntity>,
    pub after: Option<Box<EffectWrapped>>,
    pub vars: Option<Vars>,
}

impl EffectWrapped {
    pub fn push(self, context: Context, resources: &mut Resources) {
        resources.action_queue.push_back(Action {
            context,
            effect: self,
        });
    }

    pub fn process(
        &self,
        mut context: Context,
        world: &mut legion::World,
        resources: &mut Resources,
        node: &mut Option<Node>,
    ) -> Result<(), Error> {
        if context.len() > 50 {
            panic!("Too many context layers:{context}");
        }
        context.stack(
            ContextLayer::Empty {
                name: self.effect.to_string(),
            },
            world,
            resources,
        );
        let mut new_target = None;
        if let Some(target) = self.target.as_ref() {
            let target = target.calculate(&context, world, resources)?;
            new_target = Some(target);
        }
        let mut new_owner = None;
        if let Some(owner) = self.owner.as_ref() {
            let owner = owner.calculate(&context, world, resources)?;
            new_owner = Some(owner);
        }
        if let Some(target) = new_target {
            context.set_target_ref(target);
        }
        if let Some(owner) = new_owner {
            context.stack(ContextLayer::Unit { entity: owner }, world, resources);
        }
        if let Some(vars) = self.vars.as_ref() {
            context.stack(ContextLayer::Vars { vars: vars.clone() }, world, resources);
        }
        resources
            .logger
            .log(|| format!("start process {context}"), &LogContext::Effect);

        match &self.effect {
            Effect::Damage {
                value,
                on_hit: then,
            } => {
                let owner = context
                    .owner()
                    .expect(&format!("Owner not found {context}"));
                let target = context
                    .target()
                    .expect(&format!("Target not found {context}"));
                let mut value = match value {
                    Some(v) => v.calculate(&context, world, resources)?,
                    None => context
                        .get_int(&VarName::AttackValue, world)
                        .unwrap_or_default(),
                };

                context.insert_int(VarName::Damage, value);

                Event::ModifyOutgoingDamage.calculate(&mut context, world, resources);
                let initial_damage =
                    context.get_int(&VarName::Damage, world).unwrap().max(0) as usize;
                Event::BeforeOutgoingDamage {
                    owner,
                    target,
                    damage: initial_damage as usize,
                }
                .send(world, resources);
                Event::BeforeIncomingDamage {
                    owner: target,
                    caster: owner,
                    damage: initial_damage as usize,
                }
                .send(world, resources);
                let mut target_context = Context::new(
                    ContextLayer::Unit {
                        entity: context.target().unwrap(),
                    },
                    world,
                    resources,
                )
                .set_caster(context.owner().unwrap());
                target_context.insert_int(VarName::Damage, value);
                Event::ModifyIncomingDamage.calculate(&mut target_context, world, resources);
                value = target_context
                    .get_int(&VarName::Damage, world)
                    .unwrap()
                    .max(0);
                let text = format!("-{}", value);
                if let Some(node) = node.as_mut() {
                    node.add_effect(VfxSystem::vfx_show_parent_text(
                        resources,
                        &text,
                        resources.options.colors.damage,
                        resources.options.colors.deletion,
                        target,
                        1,
                        0.0,
                    ));
                }
                if value > 0 {
                    UnitSystem::deal_damage(owner, target, value as usize, world);
                    if let Some(node) = node.as_mut() {
                        node.add_effect(TimedEffect::new(
                            Some(1.0),
                            Animation::EntityShaderAnimation {
                                entity: target,
                                animation: AnimatedShaderUniforms::from_to(
                                    hashmap! {
                                        "u_damage_taken" => ShaderUniform::Float(1.0),
                                    }
                                    .into(),
                                    hashmap! {
                                        "u_damage_taken" => ShaderUniform::Float(0.0),
                                    }
                                    .into(),
                                    EasingType::Linear,
                                ),
                            },
                            0,
                        ));
                    }
                    resources.logger.log(
                        || format!("{:?} {} damage taken", target, value),
                        &LogContext::Effect,
                    );
                    if let Some(effect) = then {
                        resources.action_queue.push_front(Action::new(
                            context.clone_stack_string("after"),
                            effect.deref().clone(),
                        ));
                    }
                    Event::AfterDamageDealt {
                        owner,
                        target,
                        damage: value as usize,
                    }
                    .send(world, resources);
                }
                Event::AfterOutgoingDamage {
                    owner,
                    target,
                    damage: initial_damage as usize,
                }
                .send(world, resources);
                Event::AfterIncomingDamage {
                    owner: target,
                    caster: owner,
                    damage: initial_damage as usize,
                }
                .send(world, resources);
            }
            Effect::Heal { value } => {
                let value = value.calculate(&context, world, resources)? as usize;
                let text = format!("+{}", value);
                let target = context
                    .target()
                    .expect(&format!("Target not found {context}"));
                UnitSystem::heal_damage(
                    context
                        .owner()
                        .expect(&format!("Owner not found {context}")),
                    target,
                    value,
                    world,
                );
                if let Some(node) = node.as_mut() {
                    let color = context
                        .get_color(&VarName::Color, world)
                        .expect(&format!("Color not found {context}"));
                    node.add_effect(VfxSystem::vfx_show_parent_text(
                        resources,
                        &text,
                        resources.options.colors.healing,
                        color,
                        target,
                        0,
                        0.0,
                    ));
                }
            }
            Effect::Repeat { count, effect } => {
                for i in 0..count.calculate(&context, world, resources)? {
                    effect.process(
                        context.clone_stack_string(&format!("repeat {i}")),
                        world,
                        resources,
                        node,
                    )?;
                }
            }
            Effect::Debug { message } => debug!("Debug effect: {}", message),
            Effect::Noop => {}
            Effect::List { items } => {
                for (i, effect) in items.iter().enumerate().rev() {
                    resources.action_queue.push_front(Action::new(
                        context.clone_stack_string(&format!("list {i}")),
                        effect.deref().clone(),
                    ))
                }
            }
            Effect::SetVarInt { var, value } => {
                let value = value.calculate(&context, world, resources)?;
                context.insert_var(*var, Var::Int(value));
            }
            Effect::SetVarFaction { var, value } => {
                let value = value.calculate(&context, world, resources)?;
                context.insert_var(*var, Var::Faction(value));
            }
            Effect::ChangeStatus { name, .. }
            | Effect::AddStatus { name, .. }
            | Effect::AddTeamStatus { name, .. }
            | Effect::RemoveStatus { name, .. } => {
                let charges = match &self.effect {
                    Effect::AddStatus { name: _ } => 1,
                    Effect::AddTeamStatus { name: _ } => 1,
                    Effect::RemoveStatus { name: _ } => -1,
                    Effect::ChangeStatus { name: _, charges } => {
                        charges.calculate(&context, world, resources)?
                    }
                    _ => 0,
                };
                let target = match &self.effect {
                    Effect::AddTeamStatus { .. } => TeamSystem::entity(&Faction::Team, world)
                        .expect(&format!("Team entity not found {context}")),
                    Effect::AddStatus { .. }
                    | Effect::RemoveStatus { .. }
                    | Effect::ChangeStatus { .. } => context
                        .target()
                        .or_else(|| context.owner())
                        .expect(&format!("Target not found {context}")),
                    _ => panic!("{context}"),
                };
                Status::change_charges(target, charges, name, node, world, resources);
            }
            Effect::RemoveThisStatus => {
                let name = context
                    .get_string(&VarName::StatusName, world)
                    .expect(&format!("StatusName not found {context}"));
                let charges = context
                    .get_int(&VarName::Charges, world)
                    .expect(&format!("Charges not found {context}"));
                Status::change_charges(
                    context
                        .owner()
                        .expect(&format!("Owner not found {context}")),
                    -charges,
                    &name,
                    node,
                    world,
                    resources,
                );
            }
            Effect::ClearStatuses => {
                Status::clear_entity(
                    context
                        .target()
                        .expect(&format!("Target not found {context}")),
                    world,
                );
            }
            Effect::UseAbility {
                ability,
                force,
                charges,
            } => {
                let owner = context
                    .owner()
                    .expect(&format!("Owner not found {context}"));
                let house = &AbilityPool::get_house_origin(resources, ability);
                if !force
                    && ContextState::get(owner, world)
                        .vars
                        .try_get_house()
                        .unwrap()
                        != *house
                {
                    panic!(
                        "{} tried to use {} while not being a member of the {:?}",
                        ContextState::get(owner, world).name,
                        ability,
                        house
                    );
                }
                let mut context = context.clone_stack(
                    ContextLayer::Ability { ability: *ability },
                    world,
                    resources,
                );
                if let Some(charges) = charges {
                    context.insert_int(
                        VarName::Charges,
                        charges.calculate(&context, world, resources)?,
                    );
                }
                Event::AbilityUse {
                    ability: *ability,
                    caster: owner,
                    target: context.target().unwrap_or(owner),
                }
                .send(world, resources);
                let effect = {
                    let mut effect = Effect::ShowText {
                        text: ability.to_string(),
                        color: None,
                        entity: Some(ExpressionEntity::Owner),
                        font: 2,
                    }
                    .wrap();
                    effect.after = Some(Box::new(AbilityPool::get_effect(resources, ability)));
                    effect
                };
                effect.process(context, world, resources, node)?;
            }
            Effect::ChangeAbilityVarInt {
                ability,
                var,
                delta,
            } => {
                let delta = delta.calculate(&context, world, resources)?;
                resources.logger.log(
                    || format!("Set ability {} var {:?} delta {}", ability, var, delta),
                    &LogContext::Effect,
                );
                let ability = *ability;
                let prev_value = context
                    .stack(ContextLayer::Ability { ability }, world, resources)
                    .get_int(var, world)
                    .unwrap_or_default();
                TeamSystem::get_state_mut(
                    &context
                        .get_faction(&VarName::Faction, world)
                        .expect(&format!("Faction not found {context}")),
                    world,
                )
                .ability_vars
                .entry(ability)
                .or_default()
                .insert(*var, Var::Int(prev_value + delta));
            }
            Effect::If {
                condition,
                then,
                r#else,
            } => {
                if condition.calculate(&context, world, resources)? {
                    resources.action_queue.push_front(Action::new(
                        context.clone_stack_string("then"),
                        then.deref().clone(),
                    ));
                } else if let Some(r#else) = r#else {
                    resources.action_queue.push_front(Action::new(
                        context.clone_stack_string("else"),
                        r#else.deref().clone(),
                    ));
                }
            }
            Effect::ShowText {
                text,
                color,
                entity,
                font,
            } => {
                if let Some(node) = node.as_mut() {
                    let color = color.unwrap_or_else(|| {
                        context
                            .get_color(&VarName::Color, world)
                            .unwrap_or_else(|| {
                                context.get_color(&VarName::HouseColor, world).unwrap()
                            })
                    });

                    let mut effect = None;
                    if let Some(entity) = entity {
                        let entity = entity.calculate(&context, world, resources)?;
                        if UnitSystem::get_corpse(entity, world).is_none() {
                            effect = Some(VfxSystem::vfx_show_parent_text(
                                resources,
                                &text,
                                Rgba::WHITE,
                                color,
                                entity,
                                *font,
                                0.0,
                            ));
                        }
                    }
                    if effect.is_none() {
                        effect = Some(VfxSystem::vfx_show_text(
                            resources,
                            &text,
                            Rgba::WHITE,
                            color,
                            context.get_vec2(&VarName::Position, world).unwrap(),
                            *font,
                            0.0,
                        ));
                    }
                    node.add_effect(effect.unwrap());
                }
            }
            Effect::ShowCurve { color } => {
                let color = color
                    .or_else(|| context.get_color(&VarName::Color, world))
                    .unwrap();

                if let Some(node) = node.as_mut() {
                    node.add_effect(VfxSystem::vfx_show_curve(
                        resources,
                        context
                            .owner()
                            .expect(&format!("Owner not found {context}")),
                        context
                            .target()
                            .expect(&format!("Target not found {context}")),
                        color,
                    ));
                }
            }
            Effect::Kill => {
                UnitSystem::deal_damage(
                    context
                        .owner()
                        .expect(&format!("Owner not found {context}")),
                    context
                        .target()
                        .expect(&format!("Target not found {context}")),
                    i32::MAX as usize,
                    world,
                );
            }
            Effect::Revive { slot } => {
                let slot = slot
                    .as_ref()
                    .and_then(|x| Some(x.calculate(&context, world, resources).ok()?))
                    .unwrap_or_default() as usize;
                UnitSystem::revive_corpse(
                    context
                        .target()
                        .expect(&format!("Target not found {context}")),
                    Some(slot),
                    world,
                    &resources.logger,
                );
            }
            Effect::Aoe {
                factions,
                effect,
                exclude_self,
            } => {
                let mut faction_values: Vec<Faction> = default();
                for faction in factions {
                    faction_values.push(faction.calculate(&context, world, resources)?);
                }
                for entity in
                    UnitSystem::collect_factions(world, &HashSet::from_iter(faction_values))
                {
                    if *exclude_self && Some(entity) == context.owner() {
                        continue;
                    }
                    effect.process(
                        context
                            .clone_stack_string(&format!("Aoe {entity:?}"))
                            .set_target(entity),
                        world,
                        resources,
                        node,
                    )?;
                }
            }
            Effect::TakeVar {
                var,
                entity,
                new_name,
                effect,
            } => {
                let entity = entity.calculate(&context, world, resources)?;
                let value = Context::new(ContextLayer::Unit { entity }, world, resources)
                    .get_var(var, world)
                    .expect(&format!("Var {var} not found {context}"));
                let mut context = context.clone_stack_string(&format!("take var {var}"));
                let new_name = new_name.unwrap_or(*var);
                context.stack(
                    ContextLayer::Var {
                        var: new_name,
                        value,
                    },
                    world,
                    resources,
                );

                resources.action_queue.push_front(Action::new(
                    context.clone_stack_string(&format!("take var {var}")),
                    effect.deref().clone(),
                ));
            }
            Effect::RemoveTrigger => {
                if let Some(mut entry) = world.entry(
                    context
                        .target()
                        .expect(&format!("Target not found {context}")),
                ) {
                    entry.remove_component::<Trigger>();
                }
            }
            Effect::FindTarget {
                faction,
                condition,
                effect,
                exclude_self,
            } => {
                let faction = faction.calculate(&context, world, resources)?;
                let owner = context.owner().unwrap();
                let mut units = UnitSystem::collect_faction(world, faction);
                units.shuffle(&mut thread_rng());
                let target = units.into_iter().find(|entity| {
                    if *exclude_self && owner == *entity {
                        return false;
                    }
                    match condition.calculate(
                        &Context::new(ContextLayer::Unit { entity: *entity }, world, resources),
                        world,
                        resources,
                    ) {
                        Ok(value) => value,
                        Err(_) => false,
                    }
                });
                if let Some(target) = target {
                    effect.process(
                        context.clone_stack_string("find target").set_target(target),
                        world,
                        resources,
                        node,
                    )?
                }
            }
            Effect::AllTargets {
                faction,
                condition,
                effect,
            } => {
                let faction = faction.calculate(&context, world, resources)?;
                let targets = UnitSystem::collect_faction(world, faction)
                    .into_iter()
                    .filter_map(|entity| {
                        if let Ok(result) = condition.calculate(
                            &context.clone_stack(ContextLayer::Target { entity }, world, resources),
                            world,
                            resources,
                        ) {
                            match result {
                                true => Some(entity),
                                false => None,
                            }
                        } else {
                            None
                        }
                    })
                    .collect_vec();
                for (ind, target) in targets.into_iter().enumerate() {
                    effect.process(
                        context
                            .clone_stack_string(&format!("all targets #{ind}"))
                            .set_target(target),
                        world,
                        resources,
                        node,
                    )?;
                }
            }
            Effect::Summon { unit, slot } => {
                let slot = slot
                    .as_ref()
                    .and_then(|x| x.calculate(&context, world, resources).ok())
                    .unwrap_or_default() as usize;
                unit.unpack(world, resources, slot, None, context.owner());
                SlotSystem::fill_gaps(
                    context
                        .get_faction(&VarName::Faction, world)
                        .expect(&format!("Faction not found {context}")),
                    world,
                );
            }
            Effect::ChangeTeamVarInt {
                var,
                delta,
                faction,
            } => {
                let delta = delta.calculate(&context, world, resources)?;
                let faction = if let Some(faction) = faction {
                    faction.calculate(&context, world, resources)?
                } else {
                    context
                        .get_faction(&VarName::Faction, world)
                        .expect(&format!("Faction not found {context}"))
                };
                let state = TeamSystem::get_state_mut(&faction, world);
                state.vars.change_int(var, delta);
            }
            Effect::SetTeamVarInt {
                var,
                value,
                faction,
            } => {
                let value = value.calculate(&context, world, resources)?;
                let faction = if let Some(faction) = faction {
                    faction.calculate(&context, world, resources)?
                } else {
                    context
                        .get_faction(&VarName::Faction, world)
                        .expect(&format!("Faction not found {context}"))
                };
                let state = TeamSystem::get_state_mut(&faction, world);
                state.vars.set_int(var, value);
            }
            Effect::ChangeOwnerVarInt { var, delta } => {
                let delta = delta.calculate(&context, world, resources)?;
                let state = ContextState::get_mut(
                    context
                        .owner()
                        .expect(&format!("Owner not found {context}")),
                    world,
                );
                state.vars.change_int(var, delta);
            }
            Effect::SetOwnerVarInt { var, value } => {
                let value = value.calculate(&context, world, resources)?;
                let owner = context
                    .owner()
                    .expect(&format!("Owner not found {context}"));
                let state = ContextState::get_mut(owner, world);
                state.vars.set_int(var, value);
                resources.logger.log(
                    || format!("{owner:?} set int {var} -> {value}"),
                    &LogContext::Effect,
                );
            }
            Effect::SetOwnerVarFaction { var, value } => {
                let value = value.calculate(&context, world, resources)?;
                let owner = context
                    .owner()
                    .expect(&format!("Owner not found {context}"));
                let state = ContextState::get_mut(owner, world);
                state.vars.set_faction(var, value);
                resources.logger.log(
                    || format!("{owner:?} set faction {var} -> {value}"),
                    &LogContext::Effect,
                );
            }
        }
        Ok(match self.after.as_deref() {
            Some(after) => {
                context.stack(
                    ContextLayer::Empty {
                        name: "after".to_owned(),
                    },
                    world,
                    resources,
                );
                after.process(context, world, resources, node)?
            }
            None => (),
        })
    }
}

impl Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Effect::Repeat { count, .. } => write!(f, "{}-{}", self.as_ref(), count),
            Effect::SetVarInt { var, .. } => write!(f, "{}-{}", self.as_ref(), var),
            Effect::SetVarFaction { var, value } => {
                write!(f, "{} {} -> {}", self.as_ref(), var, value)
            }
            Effect::SetOwnerVarFaction { var, value } => {
                write!(f, "{} {} -> {}", self.as_ref(), var, value)
            }
            Effect::ChangeAbilityVarInt {
                ability,
                var,
                delta,
            } => write!(f, "{} {} {}-{}", self.as_ref(), ability, var, delta),
            Effect::ChangeTeamVarInt {
                var,
                delta,
                faction,
            } => {
                write!(f, "{} {} -> {}", self.as_ref(), var, delta)
            }
            Effect::SetTeamVarInt {
                var,
                value,
                faction,
            } => {
                write!(f, "{} {} -> {}", self.as_ref(), var, value)
            }
            Effect::ChangeOwnerVarInt { var, delta } => {
                write!(f, "{} {} -> {}", self.as_ref(), var, delta)
            }
            Effect::SetOwnerVarInt { var, value } => {
                write!(f, "{} {} -> {}", self.as_ref(), var, value)
            }
            Effect::AddStatus { name } | Effect::RemoveStatus { name } => {
                write!(f, "{} {}", self.as_ref(), name)
            }
            Effect::ChangeStatus { name, charges } => {
                write!(f, "{} {} c:{}", self.as_ref(), name, charges)
            }
            Effect::UseAbility { ability, .. } => write!(f, "{} {}", self.as_ref(), ability),
            Effect::TakeVar {
                var,
                new_name,
                entity,
                ..
            } => write!(
                f,
                "{} {} {} {}",
                self.as_ref(),
                var,
                new_name
                    .and_then(|x| Some(x.to_string()))
                    .unwrap_or_default(),
                entity
            ),
            Effect::ShowText { text, .. } => write!(f, "{} {}", self.as_ref(), text),
            Effect::Aoe { factions, .. } => {
                write!(f, "{} {}", self.as_ref(), factions.iter().join(","))
            }
            Effect::FindTarget { faction, .. } => write!(f, "{} {}", self.as_ref(), faction),
            Effect::AllTargets { faction, .. } => write!(f, "{} {}", self.as_ref(), faction),
            Effect::Summon { unit, .. } => {
                write!(f, "{} -> {unit}", self.as_ref())
            }
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}
