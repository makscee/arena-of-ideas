use super::*;

mod attack;
mod context;
mod description;
mod entity;
mod flags;
mod hp;
mod name;
mod position;
mod shader;
mod slot;
mod unit;
mod vars;

pub use attack::*;
pub use context::*;
pub use description::*;
pub use entity::*;
pub use flags::*;
pub use hp::*;
pub use name::*;
pub use position::*;
pub use shader::*;
pub use slot::*;
pub use unit::*;
pub use vars::*;

/// Components that can be deserialized from json
#[derive(Deserialize, Debug)]
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
        statuses: Vec<Status>,
    },
    Shader {
        path: PathBuf,
        parameters: Option<ShaderParameters>,
        layer: Option<ShaderLayer>,
        order: Option<i32>,
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
        entry: &mut legion::world::Entry,
        entity: &legion::Entity,
        all_statuses: &mut Statuses,
        context: &mut Context,
    ) {
        match self {
            SerializedComponent::Hp { max } => entry.add_component(HpComponent::new(*max)),
            SerializedComponent::Attack { value } => {
                entry.add_component(AttackComponent::new(*value))
            }
            SerializedComponent::StatusContainer { statuses } => {
                let mut entity_statuses = all_statuses
                    .active_statuses
                    .remove(entity)
                    .unwrap_or_default();
                for status in statuses.into_iter() {
                    all_statuses
                        .defined_statuses
                        .insert(status.name.clone(), status.clone());
                    entity_statuses.insert(
                        status.name.clone(),
                        Context {
                            status: Some((status.name.clone(), entity.clone())),
                            ..context.clone()
                        },
                    );
                }
                all_statuses
                    .active_statuses
                    .insert(entity.clone(), entity_statuses);
            }
            SerializedComponent::Shader {
                path,
                parameters,
                layer,
                order,
            } => entry.add_component(Shader {
                path: path.clone(),
                parameters: parameters.clone().unwrap_or_default(),
                layer: layer.clone().unwrap_or(ShaderLayer::Unit),
                order: order.unwrap_or_default(),
            }),
            SerializedComponent::Name { name } => entry.add_component(Name::new(name)),
            SerializedComponent::Description { text } => {
                entry.add_component(Description::new(text))
            }
        }
    }
}
