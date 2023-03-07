use geng::prelude::itertools::Itertools;

use super::*;

pub struct Cassette {
    pub head: Time,
    queue: Vec<CassetteNode>,
    pub node_template: CassetteNode, // any new node will be cloned from this
    pub parallel_node: CassetteNode, // this node is always rendered
}

impl Default for Cassette {
    fn default() -> Self {
        Self {
            head: default(),
            queue: vec![default()],
            node_template: default(),
            parallel_node: default(),
        }
    }
}

const DEFAULT_EFFECT_KEY: &str = "default";

impl Cassette {
    pub fn close_node(&mut self) {
        let node = self.queue.last_mut().unwrap();
        let start = (node.start + node.duration).max(self.head);
        if node.duration == 0.0 {
            node.start = start;
            self.queue.pop();
        }
        let mut new_node = self.node_template.clone();
        new_node.start = start;
        self.queue.push(new_node);
    }

    pub fn merge_template_into_last(&mut self) {
        let node = self.queue.last_mut().unwrap();
        node.merge_mut(&self.node_template);
    }

    pub fn add_effect(&mut self, effect: VisualEffect) {
        self.add_effect_by_key(DEFAULT_EFFECT_KEY, effect);
    }

    pub fn add_effect_by_key(&mut self, key: &str, mut effect: VisualEffect) {
        let mut last = self.queue.last_mut().unwrap();
        if self.head > last.start + last.duration {
            self.close_node();
            last = self.queue.last_mut().unwrap();
        }
        if self.head > last.start && self.head < last.start + last.duration {
            effect.delay += self.head - last.start;
        }
        last.add_effect_by_key(key, effect);
    }

    pub fn get_key_count(&self, key: &str) -> usize {
        self.queue.last().unwrap().get_key_count(key)
    }

    pub fn add_entity_shader(&mut self, entity: legion::Entity, shader: Shader) {
        self.queue
            .last_mut()
            .unwrap()
            .add_entity_shader(entity, shader);
    }

    pub fn get_shaders(
        &self,
        mouse_pos: vec2<f32>,
        mut world_shaders: HashMap<legion::Entity, Shader>,
    ) -> Vec<Shader> {
        let node = self.get_node_at_ts(self.head).merge(&self.parallel_node);
        let time = self.head - node.start;
        let mut shaders: Vec<Shader> = default();
        world_shaders.extend(node.entity_shaders.clone().into_iter());
        let mut entity_shaders = world_shaders;

        // 1st phase: apply any changes to entity shaders uniforms
        for effect in node.effects.values().flatten().sorted_by_key(|x| x.order) {
            let time = time - effect.delay;
            if effect.duration > 0.0 && (time > effect.duration || time < 0.0) {
                continue;
            }
            let effect_type = &effect.r#type;
            match effect_type {
                VisualEffectType::EntityShaderAnimation { .. }
                | VisualEffectType::EntityShaderConst { .. } => {
                    effect_type.process(time / effect.duration, &mut entity_shaders);
                }
                _ => {}
            };
        }

        // todo: rework
        // inject hovered info
        for (_, shader) in entity_shaders.iter_mut() {
            let position = shader
                .parameters
                .uniforms
                .get(&VarName::Position.convert_to_uniform())
                .and_then(|x| match x {
                    ShaderUniform::Vec2(v) => Some(v),
                    _ => None,
                });
            let radius = shader
                .parameters
                .uniforms
                .get(&VarName::Radius.convert_to_uniform())
                .and_then(|x| match x {
                    ShaderUniform::Float(v) => Some(v),
                    _ => None,
                });
            if position.is_none() || radius.is_none() {
                continue;
            }
            let position = position.unwrap();
            let radius = radius.unwrap();
            if (mouse_pos - *position).len() < *radius {
                shader
                    .parameters
                    .uniforms
                    .insert("u_hovered".to_string(), ShaderUniform::Float(1.0));
            }
        }

        // 2nd phase: apply any other shaders that might need updated entity shaders uniforms
        for effect in node.effects.values().flatten().sorted_by_key(|x| x.order) {
            let time = time - effect.delay;
            if effect.duration > 0.0 && (time > effect.duration || time < 0.0) {
                continue;
            }
            let effect_type = &effect.r#type;
            match effect_type {
                VisualEffectType::EntityShaderAnimation { .. }
                | VisualEffectType::EntityShaderConst { .. } => {}
                _ => {
                    shaders
                        .extend(effect_type.process(time / effect.duration, &mut entity_shaders));
                }
            };
        }
        [
            entity_shaders
                .into_iter()
                .sorted_by_key(|item| format!("{:?}", item.0))
                .map(|item| item.1)
                .collect_vec(),
            shaders,
        ]
        .concat()
    }

    pub fn length(&self) -> Time {
        let last = self.queue.last().unwrap();
        last.start + last.duration
    }

    pub fn last_start(&self) -> Time {
        self.queue.last().unwrap().start
    }

    pub fn clear(&mut self) {
        self.queue = vec![default()];
        self.head = 0.0;
        self.node_template.clear();
        self.parallel_node.clear();
    }

    fn get_node_at_ts(&self, ts: Time) -> &CassetteNode {
        if ts > self.length() {
            return &self.node_template;
        }
        let index = match self
            .queue
            .binary_search_by_key(&r32(ts), |node| r32(node.start))
        {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        if let Some(node) = self.queue.get(index) {
            node
        } else {
            &self.node_template
        }
    }
}

#[derive(Default, Clone, Debug)]

pub struct CassetteNode {
    start: Time,
    duration: Time,
    entity_shaders: HashMap<legion::Entity, Shader>,
    effects: HashMap<String, Vec<VisualEffect>>,
}

impl CassetteNode {
    pub fn add_entity_shader(&mut self, entity: legion::Entity, shader: Shader) {
        self.entity_shaders.insert(entity, shader);
    }
    pub fn add_effect_by_key(&mut self, key: &str, effect: VisualEffect) {
        self.duration = self.duration.max(effect.duration + effect.delay);
        let mut vec = self.effects.remove(key).unwrap_or_default();
        vec.push(effect);
        self.effects.insert(key.to_string(), vec);
    }
    pub fn add_effects_by_key(&mut self, key: &str, effects: Vec<VisualEffect>) {
        effects
            .into_iter()
            .for_each(|effect| self.add_effect_by_key(key, effect))
    }
    pub fn get_key_count(&self, key: &str) -> usize {
        match self.effects.get(key).and_then(|v| Some(v.len())) {
            Some(value) => value,
            None => 0,
        }
    }
    pub fn clear_key(&mut self, key: &str) {
        self.effects.remove(key);
    }
    pub fn clear(&mut self) {
        self.start = default();
        self.duration = default();
        self.entity_shaders.clear();
        self.effects.clear();
    }
    pub fn clear_entities(&mut self) {
        self.entity_shaders.clear();
    }
    pub fn merge(&self, other: &CassetteNode) -> CassetteNode {
        let mut node = self.clone();
        node.merge_mut(other);
        node
    }
    pub fn merge_mut(&mut self, other: &CassetteNode) {
        let mut node = self;
        node.duration = node.duration.max(node.duration);
        for (key, other_effects) in other.effects.iter() {
            if key == DEFAULT_EFFECT_KEY {
                let mut effects = node.effects.remove(key).unwrap_or_default();
                effects.extend(other_effects.iter().cloned());
                node.effects.insert(key.clone(), effects);
            } else {
                node.effects.insert(key.clone(), other_effects.clone());
            }
        }
        other.entity_shaders.iter().for_each(|(entity, shader)| {
            node.entity_shaders.insert(*entity, shader.clone());
        });
    }
    pub fn start(&self) -> Time {
        self.start
    }
}
