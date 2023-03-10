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
    pub new_statuses: VecDeque<(legion::Entity, String)>, // statuses that need to be inited
}

impl StatusPool {
    pub fn collect_triggers(&self, entity: &legion::Entity) -> Vec<(Trigger, Context)> {
        self.collect_statuses(entity)
            .into_iter()
            .map(|(status, context)| (status.trigger, context))
            .collect_vec()
    }

    pub fn collect_statuses(&self, entity: &legion::Entity) -> Vec<(Status, Context)> {
        self.active_statuses
            .get(entity)
            .unwrap_or(&default())
            .iter()
            .map(|(name, context)| {
                (
                    self.defined_statuses
                        .get(name)
                        .expect(&format!("Status undefined: {}", name))
                        .clone(),
                    context.clone(),
                )
            })
            .collect_vec()
    }

    pub fn get_entity_shaders(&self, entity: &legion::Entity) -> Vec<Shader> {
        self.collect_statuses(entity)
            .into_iter()
            .filter_map(|(status, _)| match status.shader {
                Some(shader) => Some(shader),
                None => None,
            })
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
                            .shaders
                            .description_panel
                            .clone()
                            .set_uniform("u_offset", ShaderUniform::Vec2(vec2(0.0, -2.0)))
                            .set_uniform("u_description", ShaderUniform::String((1, text.clone())))
                            .set_uniform("u_name", ShaderUniform::String((2, key.clone())))
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
        entity: legion::Entity,
        status_name: &str,
        context: Context,
        resources: &mut Resources,
    ) {
        let mut statuses = resources
            .status_pool
            .active_statuses
            .remove(&entity)
            .unwrap_or_default();
        statuses.insert(status_name.to_string(), context.clone());
        resources
            .status_pool
            .active_statuses
            .insert(entity, statuses);
        resources
            .status_pool
            .new_statuses
            .push_back((entity, status_name.to_string()));
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

    pub fn init_new_statuses(world: &legion::World, resources: &mut Resources) {
        while let Some((entity, status_name)) = resources.status_pool.new_statuses.pop_front() {
            let mut context = ContextSystem::get_context(entity, world);
            context.vars.merge(
                &resources
                    .status_pool
                    .active_statuses
                    .get(&entity)
                    .unwrap()
                    .get(&status_name)
                    .unwrap()
                    .vars,
                true,
            );
            Event::Init {
                status: status_name.to_string(),
                context,
            }
            .send(resources, world);
        }
    }

    pub fn clear_all_active(&mut self) {
        self.active_statuses.clear();
    }

    pub fn clear_entity(&mut self, entity: &legion::Entity) {
        self.active_statuses.remove(entity);
    }
}
