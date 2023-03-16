use super::*;
use geng::prelude::itertools::Itertools;

mod ability_description;
mod area;
mod attack;
mod context;
mod description;
mod entity;
mod flags;
mod house;
mod hp;
mod name;
mod shader;
mod slot;
mod unit;
mod vars;
mod world;

pub use ability_description::*;
pub use area::*;
pub use attack::*;
pub use context::*;
pub use description::*;
pub use entity::*;
pub use flags::*;
pub use house::*;
pub use hp::*;
pub use name::*;
pub use shader::*;
pub use slot::*;
pub use unit::*;
pub use vars::*;
pub use world::*;

/// Components that can be deserialized from json
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "component")]
pub enum SerializedComponent {
    Name {
        name: String,
    },
    Description {
        text: String,
    },
    Hp {
        max: Hp,
    },
    Attack {
        value: Hp,
    },
    StatusContainer {
        statuses: HashMap<String, Status>,
    },
    Shader {
        path: PathBuf,
        parameters: Option<ShaderParameters>,
        layer: Option<ShaderLayer>,
        order: Option<i32>,
        #[serde(default)]
        chain: Box<Vec<SerializedComponent>>,
    },
    House {
        houses: Vec<HouseName>,
    },
    AbilityDescription {
        abilities: Vec<String>,
    },
}

impl fmt::Display for SerializedComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl SerializedComponent {
    pub fn add_to_entry(
        &self,
        resources: &mut Resources,
        path: &PathBuf,
        entity: legion::Entity,
        context: &Context,
        world: &mut legion::World,
    ) {
        let mut entry = world.entry(entity).unwrap();
        match self {
            SerializedComponent::Hp { max } => entry.add_component(HpComponent::new(*max)),
            SerializedComponent::Attack { value } => {
                entry.add_component(AttackComponent::new(*value))
            }
            SerializedComponent::StatusContainer { statuses } => {
                statuses.into_iter().for_each(|(name, status)| {
                    let name = format!("{}_{}", path.file_name().unwrap().to_string_lossy(), name);
                    resources
                        .status_pool
                        .define_status(name.clone(), status.clone());
                    StatusPool::add_entity_status(
                        entity,
                        &name,
                        Context {
                            parent: Some(context.owner),
                            ..context.clone()
                        },
                        resources,
                    );
                });
            }
            SerializedComponent::Shader { .. } => {
                let shader = Shader {
                    chain_before: Box::new(Self::unpack_shader(self).into_iter().collect_vec()),
                    ..resources.options.shaders.unit.clone()
                };
                entry.add_component(shader)
            }

            SerializedComponent::Name { name } => entry.add_component(NameComponent::new(name)),
            SerializedComponent::Description { text } => {
                entry.add_component(DescriptionComponent::new(text))
            }
            SerializedComponent::House { houses } => {
                entry.add_component(HouseComponent::new(houses.clone(), &resources))
            }
            SerializedComponent::AbilityDescription { abilities } => entry.add_component(
                AbilityDescriptionComponent::new(abilities, &resources.houses),
            ),
        }
    }

    pub fn unpack_shader(component: &SerializedComponent) -> Option<Shader> {
        match component {
            SerializedComponent::Shader {
                path,
                parameters,
                layer,
                order,
                chain,
            } => {
                let chain = Box::new(
                    chain
                        .deref()
                        .into_iter()
                        .map(|shader| Self::unpack_shader(shader).unwrap())
                        .collect_vec(),
                );
                return Some(Shader {
                    path: path.clone(),
                    parameters: parameters.clone().unwrap_or_default(),
                    layer: layer.clone().unwrap_or(ShaderLayer::Unit),
                    order: order.unwrap_or_default(),
                    chain_before: chain,
                    chain_after: default(),
                });
            }
            _ => None,
        }
    }
}
