use super::*;

mod attack;
mod context;
mod description;
mod drag;
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
pub use drag::*;
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
        chain: Option<Box<SerializedComponent>>,
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
                statuses.into_iter().for_each(|status| {
                    all_statuses.add_and_define_entity_status(entity, &status, context.clone())
                });
            }
            SerializedComponent::Shader {
                path,
                parameters,
                layer,
                order,
                chain,
            } => entry.add_component(*Self::get_shader(self).unwrap()),

            SerializedComponent::Name { name } => entry.add_component(NameComponent::new(name)),
            SerializedComponent::Description { text } => {
                entry.add_component(DescriptionComponent::new(text))
            }
        }
    }

    fn get_shader(component: &SerializedComponent) -> Option<Box<Shader>> {
        match component {
            SerializedComponent::Shader {
                path,
                parameters,
                layer,
                order,
                chain,
            } => Some(Box::new(Shader {
                path: path.clone(),
                parameters: parameters.clone().unwrap_or_default(),
                layer: layer.clone().unwrap_or(ShaderLayer::Unit),
                order: order.unwrap_or_default(),
                chain: chain
                    .as_ref()
                    .and_then(|component| Self::get_shader(&**component)),
            })),
            _ => None,
        }
    }
}
