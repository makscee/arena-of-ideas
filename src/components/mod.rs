use super::*;

mod attack;
mod context;
mod flags;
mod game_state;
mod hp;
mod position;
mod shader;
mod unit;
mod vars;

pub use attack::*;
pub use context::*;
pub use flags::*;
pub use game_state::*;
pub use hp::*;
pub use position::*;
pub use shader::*;
pub use unit::*;
pub use vars::*;

/// Components that can be deserialized from json
#[derive(Deserialize, Debug)]
#[serde(tag = "component")]
pub enum Component {
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

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Component {
    pub fn add_to_entry(
        &self,
        entry: &mut legion::world::Entry,
        entity: &legion::Entity,
        all_statuses: &mut Statuses,
        context: &mut Context,
    ) {
        match self {
            Component::Hp { max } => entry.add_component(HpComponent::new(context, *max)),
            Component::Attack { value } => entry.add_component(AttackComponent::new(*value)),
            Component::StatusContainer { statuses } => {
                let mut entity_statuses = all_statuses
                    .active_statuses
                    .remove(entity)
                    .unwrap_or_default();
                let context = Context {
                    owner: entity.clone(),
                    target: entity.clone(),
                    creator: entity.clone(),
                    vars: default(),
                    status: default(),
                };
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
            Component::Shader {
                path,
                parameters,
                layer,
                order,
            } => entry.add_component(Shader {
                path: static_path().join(path),
                parameters: parameters.clone().unwrap_or_default(),
                layer: layer.clone().unwrap_or(ShaderLayer::Unit),
                order: order.unwrap_or_default(),
            }),
        }
    }
}
