use strum_macros::Display;

/// Expression is anything that can return a value.
/// For each return type there should be one enum
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ExpressionInt {
    Sum {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Sub {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Mul {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Div {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Max {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    If {
        condition: Box<Condition>,
        then: Box<ExpressionInt>,
        r#else: Box<ExpressionInt>,
    },
    Const {
        value: i32,
    },
    Var {
        var: VarName,
    },
    EntityVar {
        var: VarName,
        entity: ExpressionEntity,
    },
    AbilityVar {
        ability: AbilityName,
        var: VarName,
    },
    TeamVar {
        var: VarName,
    },
    Negate {
        value: Box<ExpressionInt>,
    },
    StatusCharges {
        name: String,
    },
    Faction {
        faction: ExpressionFaction,
    },
}

impl ExpressionInt {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Result<i32, Error> {
        resources.logger.log(
            || {
                format!(
                    "Calculating int expression {:?} o:{:?} t:{:?}",
                    self,
                    context.owner(),
                    context.target()
                )
            },
            &LogContext::Expression,
        );
        let result =
            match self {
                ExpressionInt::Sum { a, b } => Ok(a.calculate(context, world, resources)?
                    + b.calculate(context, world, resources)?),
                ExpressionInt::Sub { a, b } => Ok(a.calculate(context, world, resources)?
                    - b.calculate(context, world, resources)?),
                ExpressionInt::Mul { a, b } => Ok(a.calculate(context, world, resources)?
                    * b.calculate(context, world, resources)?),
                ExpressionInt::Div { a, b } => Ok(a.calculate(context, world, resources)?
                    / b.calculate(context, world, resources)?),
                ExpressionInt::Max { a, b } => Ok(a
                    .calculate(context, world, resources)?
                    .max(b.calculate(context, world, resources)?)),
                ExpressionInt::Const { value } => Ok(*value),
                ExpressionInt::Var { var } => context
                    .get_int(var, world)
                    .context(format!("Failed to find var {var}")),
                ExpressionInt::EntityVar { var, entity } => {
                    let context = Context::new(
                        ContextLayer::Unit {
                            entity: entity.calculate(context, world, resources)?,
                        },
                        world,
                        resources,
                    );
                    let value = context.get_int(var, world);
                    dbg!(&value);
                    value.context(format!("Var not found {var}"))
                }
                ExpressionInt::AbilityVar { ability, var } => {
                    let ability = *ability;
                    context
                        .clone_stack(ContextLayer::Ability { ability }, world, resources)
                        .get_int(var, world)
                        .context("Failed to get ability var")
                }
                ExpressionInt::TeamVar { var } => {
                    let faction = context.get_faction(&VarName::Faction, world).unwrap();
                    TeamSystem::get_state(faction, world)
                        .vars
                        .try_get_int(var)
                        .context("Failed to get team var")
                }
                ExpressionInt::If {
                    condition,
                    then,
                    r#else,
                } => match condition.calculate(context, world, resources)? {
                    true => then.calculate(context, world, resources),
                    false => r#else.calculate(context, world, resources),
                },
                ExpressionInt::Negate { value } => Ok(-value.calculate(context, world, resources)?),
                ExpressionInt::StatusCharges { name } => {
                    Ok(context.get_status_charges(name, world))
                }
                ExpressionInt::Faction { faction } => Ok(UnitSystem::collect_faction(
                    world,
                    faction.calculate(context, world, resources)?,
                )
                .len() as i32),
            };

        resources
            .logger
            .log(|| format!("Result {result:?}",), &LogContext::Expression);
        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(tag = "type")]
pub enum ExpressionEntity {
    World,
    Target,
    Caster,
    Owner,
    FindUnit {
        slot: Box<ExpressionInt>,
        faction: ExpressionFaction,
    },
    RandomUnit {
        faction: ExpressionFaction,
        #[serde(default)]
        skip_self: bool,
    },
    SlotRelative {
        relation: Box<ExpressionInt>,
    },
    Entity {
        entity: legion::Entity,
    },
    Var {
        var: VarName,
    },
}

impl Default for ExpressionEntity {
    fn default() -> Self {
        Self::Owner
    }
}

impl ExpressionEntity {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Result<legion::Entity, Error> {
        resources.logger.log(
            || {
                format!(
                    "Calculating entity expression {:?} o:{:?} t:{:?}",
                    self,
                    context.owner(),
                    context.target()
                )
            },
            &LogContext::Expression,
        );
        let result = match self {
            ExpressionEntity::World => Ok(<(&WorldComponent, &EntityComponent)>::query()
                .iter(world)
                .next()
                .unwrap()
                .1
                .entity),
            ExpressionEntity::Target => context.target().context("No target"),
            ExpressionEntity::Caster => context.caster().context("No target"),
            ExpressionEntity::Owner => context.owner().context("No owner"),
            ExpressionEntity::FindUnit { slot, faction } => {
                let slot = slot.calculate(context, world, resources)? as usize;
                let faction = faction.calculate(context, world, resources)?;
                SlotSystem::find_unit_by_slot(slot, &faction, world)
                    .context(format!("No unit of {faction:?} found in {slot} slot"))
            }
            ExpressionEntity::RandomUnit { faction, skip_self } => {
                let faction = faction.calculate(context, world, resources)?;
                UnitSystem::collect_faction(world, faction)
                    .into_iter()
                    .filter(|x| !skip_self || Some(*x) != context.owner())
                    .choose(&mut thread_rng())
                    .context(format!("No units of {faction:?} found"))
            }
            ExpressionEntity::SlotRelative { relation } => {
                let faction = context.get_faction(&VarName::Faction, world).unwrap();
                let slot = (context.get_int(&VarName::Slot, world).unwrap()
                    + relation.calculate(context, world, resources)?)
                    as usize;
                SlotSystem::find_unit_by_slot(slot, &faction, world)
                    .context(format!("No unit of {faction:?} found in slot {slot}"))
            }
            ExpressionEntity::Entity { entity } => Ok(*entity),
            ExpressionEntity::Var { var } => context
                .get_entity(var, world)
                .context(format!("Failed to get entity from {var:?}")),
        };

        resources
            .logger
            .log(|| format!("Result {result:?}",), &LogContext::Expression);
        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(tag = "type")]
pub enum ExpressionFaction {
    Owner,
    Target,
    Caster,
    Opposite { faction: Box<ExpressionFaction> },
    Var { var: VarName },
    Team,
    Shop,
    Light,
    Dark,
}

impl Default for ExpressionFaction {
    fn default() -> Self {
        Self::Owner
    }
}

impl ExpressionFaction {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Result<Faction, Error> {
        resources.logger.log(
            || {
                format!(
                    "Calculating faction expression {:?} o:{:?} t:{:?}",
                    self,
                    context.owner(),
                    context.target()
                )
            },
            &LogContext::Expression,
        );
        let result = match &self {
            ExpressionFaction::Owner => context
                .get_faction(&VarName::Faction, world)
                .context("Failed to get faction"),
            ExpressionFaction::Target => context
                .clone_stack(
                    ContextLayer::Unit {
                        entity: context.target().unwrap(),
                    },
                    world,
                    resources,
                )
                .get_faction(&VarName::Faction, world)
                .context("Failed to get faction"),
            ExpressionFaction::Caster => context
                .clone_stack(
                    ContextLayer::Unit {
                        entity: context.caster().unwrap(),
                    },
                    world,
                    resources,
                )
                .get_faction(&VarName::Faction, world)
                .context("Failed to get faction"),
            ExpressionFaction::Opposite { faction } => {
                Ok(faction.calculate(context, world, resources)?.opposite())
            }
            ExpressionFaction::Var { var } => context
                .get_faction(var, world)
                .context("Failed to get faction var"),
            ExpressionFaction::Team => Ok(Faction::Team),
            ExpressionFaction::Shop => Ok(Faction::Shop),
            ExpressionFaction::Light => Ok(Faction::Light),
            ExpressionFaction::Dark => Ok(Faction::Dark),
        };

        resources
            .logger
            .log(|| format!("Result {result:?}",), &LogContext::Expression);
        result
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ExpressionUniform {
    Uniform {
        key: String,
    },
    Float {
        value: f32,
    },
    GlobalTime,
    Vec2 {
        x: Box<ExpressionUniform>,
        y: Box<ExpressionUniform>,
    },
    Vec4 {
        x: Box<ExpressionUniform>,
        y: Box<ExpressionUniform>,
        z: Box<ExpressionUniform>,
        w: Box<ExpressionUniform>,
    },
    Color {
        r: Box<ExpressionUniform>,
        g: Box<ExpressionUniform>,
        b: Box<ExpressionUniform>,
        a: Box<ExpressionUniform>,
    },
    Sum {
        a: Box<ExpressionUniform>,
        b: Box<ExpressionUniform>,
    },
    Mul {
        a: Box<ExpressionUniform>,
        b: Box<ExpressionUniform>,
    },
    Sin {
        x: Box<ExpressionUniform>,
    },
}

impl ExpressionUniform {
    pub fn calculate(&self, uniforms: &ShaderUniforms) -> ShaderUniform {
        match self {
            ExpressionUniform::Uniform { key } => uniforms.get(key).expect(&format!(
                "Failed to get Uniform expression for key {key} {uniforms:?}"
            )),
            ExpressionUniform::Sum { a, b } => a.calculate(uniforms).sum(&b.calculate(uniforms)),
            ExpressionUniform::Mul { a, b } => a.calculate(uniforms).mul(&b.calculate(uniforms)),
            ExpressionUniform::Float { value } => ShaderUniform::Float(*value),
            ExpressionUniform::Sin { x } => {
                ShaderUniform::Float(x.calculate(uniforms).unpack_float().sin())
            }
            ExpressionUniform::GlobalTime => ShaderUniform::Float(global_time()),
            ExpressionUniform::Vec2 { x, y } => ShaderUniform::Vec2(vec2(
                x.calculate(uniforms).unpack_float(),
                y.calculate(uniforms).unpack_float(),
            )),
            ExpressionUniform::Vec4 { x, y, z, w } => ShaderUniform::Vec4(vec4(
                x.calculate(uniforms).unpack_float(),
                y.calculate(uniforms).unpack_float(),
                z.calculate(uniforms).unpack_float(),
                w.calculate(uniforms).unpack_float(),
            )),
            ExpressionUniform::Color { r, g, b, a } => ShaderUniform::Color(Rgba::new(
                r.calculate(uniforms).unpack_float(),
                g.calculate(uniforms).unpack_float(),
                b.calculate(uniforms).unpack_float(),
                a.calculate(uniforms).unpack_float(),
            )),
        }
    }
}
