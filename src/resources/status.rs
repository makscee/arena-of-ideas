use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Status {
    pub trigger: Trigger,
    pub description: Option<String>,
    pub color: Option<Rgba<f32>>,
    pub shader: Option<Shader>,
}

#[derive(Default, Debug)]
pub struct StatusPool {
    pub defined_statuses: HashMap<String, Status>, // key = status name
    pub active_statuses: HashMap<legion::Entity, HashMap<String, Context>>, // entity -> status name -> context
}

impl StatusPool {
    pub fn get_entity_shaders(&self, entity: &legion::Entity) -> Vec<Shader> {
        self.active_statuses
            .get(entity)
            .unwrap_or(&default())
            .keys()
            .filter_map(|key| self.defined_statuses.get(key).unwrap().shader.clone())
            .collect_vec()
    }

    pub fn get_description_shaders(
        &self,
        entity: &legion::Entity,
        options: &Options,
    ) -> Vec<Shader> {
        self.active_statuses
            .get(entity)
            .unwrap_or(&default())
            .keys()
            .filter_map(|key| {
                let status = self.defined_statuses.get(key).unwrap();
                match &status.description {
                    Some(text) => Some(
                        options
                            .status_description
                            .clone()
                            .set_uniform("u_description", ShaderUniform::String((1, text.clone())))
                            .set_uniform("u_name", ShaderUniform::String((0, key.clone())))
                            .set_uniform(
                                "u_color",
                                ShaderUniform::Color(match status.color {
                                    Some(color) => color,
                                    None => Rgba::BLACK,
                                }),
                            ),
                    ),
                    _ => None,
                }
            })
            .collect_vec()
    }

    pub fn add_entity_status(
        entity: &legion::Entity,
        status_name: &str,
        context: Context,
        resources: &mut Resources,
    ) {
        let mut statuses = resources
            .status_pool
            .active_statuses
            .remove(entity)
            .unwrap_or_default();
        statuses.insert(
            status_name.to_string(),
            Context {
                status: Some((status_name.to_string(), entity.clone())),
                ..context.clone()
            },
        );
        resources
            .status_pool
            .active_statuses
            .insert(*entity, statuses);

        {
            Event::Init {
                status: status_name.to_string(),
            }
        }
        .send(&context, resources);
    }

    pub fn define_status(&mut self, name: String, mut status: Status) {
        if status.shader.is_some() && status.color.is_some() {
            if let Some(ref mut shader) = status.shader {
                shader.parameters.uniforms.insert(
                    "u_color".to_string(),
                    ShaderUniform::Color(status.color.unwrap()),
                );
            }
        }
        self.defined_statuses.insert(name, status);
    }
}
