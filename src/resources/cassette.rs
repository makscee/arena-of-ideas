use geng::prelude::itertools::Itertools;

use super::*;

pub struct Cassette {
    pub head: Time,
    queue: Vec<CassetteNode>,
    pub node_template: CassetteNode, // any new node will be cloned from this
}

impl Default for Cassette {
    fn default() -> Self {
        Self {
            head: default(),
            queue: vec![default()],
            node_template: default(),
        }
    }
}

impl Cassette {
    pub fn close_node(&mut self) {
        let node = self.queue.last().unwrap();
        let start = node.start + node.duration;
        if node.duration == 0.0 {
            self.queue.pop();
        }
        let mut new_node = self.node_template.clone();
        new_node.start = start;
        self.queue.push(new_node);
    }

    pub fn add_effect(&mut self, effect: VisualEffect) {
        self.queue.last_mut().unwrap().add_effect(effect);
    }

    pub fn add_entity_shader(&mut self, entity: legion::Entity, shader: Shader) {
        self.queue
            .last_mut()
            .unwrap()
            .add_entity_shader(entity, shader);
    }

    pub fn get_shaders(&self) -> Vec<Shader> {
        let node = self.get_node_at_ts(self.head);
        let time = self.head - node.start;
        let mut shaders: Vec<Shader> = default();
        let mut entity_shaders = node.entity_shaders.clone();
        for effect in node.effects.iter() {
            if effect.duration > 0.0 && time > effect.duration {
                continue;
            }
            shaders.extend(
                effect
                    .r#type
                    .process(time / effect.duration, &mut entity_shaders),
            );
        }
        let mut shaders = [entity_shaders.into_values().collect_vec(), shaders].concat();
        shaders.iter_mut().for_each(|shader| {
            shader
                .parameters
                .uniforms
                .insert("u_game_time".to_string(), ShaderUniform::Float(self.head))
        });

        shaders
    }

    pub fn length(&self) -> Time {
        let last = self.queue.last().unwrap();
        last.start + last.duration
    }

    pub fn last_start(&self) -> Time {
        self.queue.last().unwrap().start
    }

    pub fn get_skip_ts(&self, from_ts: Time, right: bool) -> Time {
        let node = self.get_node_at_ts(
            from_ts
                + match right {
                    true => 0.001,
                    false => -0.001,
                },
        );
        if right {
            node.start + node.duration
        } else {
            node.start
        }
    }

    fn get_node_at_ts(&self, ts: Time) -> &CassetteNode {
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
            &self.queue.last().unwrap()
        }
    }
}

#[derive(Default, Clone, Debug)]

pub struct CassetteNode {
    start: Time,
    duration: Time,
    entity_shaders: HashMap<legion::Entity, Shader>,
    effects: Vec<VisualEffect>,
}

impl CassetteNode {
    pub fn add_entity_shader(&mut self, entity: legion::Entity, shader: Shader) {
        self.entity_shaders.insert(entity, shader);
    }
    pub fn add_effect(&mut self, effect: VisualEffect) {
        self.duration = self.duration.max(effect.duration);
        self.effects.push(effect);
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
}
