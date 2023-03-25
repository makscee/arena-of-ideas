use geng::prelude::itertools::Itertools;

use super::*;

pub struct Cassette {
    pub head: Time,
    tape: Vec<CassetteNode>,
    pub render_node: CassetteNode, // this node is always rendered
}

impl Default for Cassette {
    fn default() -> Self {
        Self {
            render_node: default(),
            tape: vec![default()],
            head: default(),
        }
    }
}

const DEFAULT_EFFECT_KEY: &str = "default";

impl Cassette {
    pub fn add_tape_nodes(&mut self, mut nodes: Vec<CassetteNode>) {
        let head = self.head;
        let last = self.last_mut();
        if last.start + last.duration < head {
            last.duration = head - last.start;
        }
        nodes.drain(..).for_each(|mut node| {
            let start = self.last().start + self.last().duration;
            node.start = start;
            self.tape.push(node);
        })
    }

    pub fn clear_tape(&mut self) {
        self.tape = vec![default()];
    }

    pub fn get_key_count(&self, key: &str) -> usize {
        self.last().get_key_count(key)
    }

    pub fn get_shaders(
        resources: &mut Resources,
        mut world_shaders: HashMap<legion::Entity, Shader>,
    ) -> Vec<Shader> {
        let cassette = &resources.cassette;
        let mut node = cassette
            .get_node_at_ts(cassette.head)
            .and_then(|x| Some(x.merge(&cassette.render_node)))
            .or(Some(cassette.render_node.clone()))
            .unwrap();
        let time = cassette.head - node.start;
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
        UnitSystem::inject_entity_shaders_uniforms(&mut entity_shaders, resources);
        StatusSystem::add_active_statuses_panel_to_node(&mut node, resources);

        // 2nd phase: apply any other shaders that might need updated entity shaders uniforms
        let mut extra_shaders: Vec<Shader> = default();
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
                    extra_shaders
                        .extend(effect_type.process(time / effect.duration, &mut entity_shaders));
                }
            };
        }

        let mut entity_shaders_vec = entity_shaders
            .into_iter()
            .sorted_by_key(|(entity, shader)| {
                (shader.layer.index(), shader.order, format!("{:?}", entity))
            })
            .collect_vec();

        let mut hovered_entity = None;
        for (entity, shader) in entity_shaders_vec.iter().rev() {
            if let Some(area) = AreaComponent::from_shader(shader) {
                if area.contains(resources.input.mouse_pos) {
                    hovered_entity = Some(*entity);
                    break;
                }
            }
        }
        if let Some(hovered) = InputSystem::set_hovered_entity(hovered_entity, resources) {
            let last_ind = entity_shaders_vec.len() - 1;
            if let Some(hovered_ind) = entity_shaders_vec.iter().position(|x| x.0 == hovered) {
                entity_shaders_vec.swap(hovered_ind, last_ind);
            }
        }
        let entity_shaders_vec = entity_shaders_vec
            .into_iter()
            .map(|(_, shader)| shader)
            .collect_vec();

        [entity_shaders_vec, extra_shaders].concat()
    }

    pub fn length(&self) -> Time {
        let last = self.last();
        last.start + last.duration
    }

    pub fn clear(&mut self) {
        self.clear_tape();
        self.head = 0.0;
        self.render_node.clear();
    }

    pub fn last_mut(&mut self) -> &mut CassetteNode {
        self.tape.last_mut().unwrap()
    }

    pub fn last(&self) -> &CassetteNode {
        self.tape.last().unwrap()
    }

    fn get_node_at_ts(&self, ts: Time) -> Option<&CassetteNode> {
        if ts > self.length() {
            return None;
        }
        let index = match self
            .tape
            .binary_search_by_key(&r32(ts), |node| r32(node.start))
        {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        self.tape.get(index)
    }
}

#[derive(Default, Clone, Debug)]

pub struct CassetteNode {
    pub start: Time,
    pub duration: Time,
    pub entity_shaders: HashMap<legion::Entity, Shader>,
    active_statuses: HashMap<legion::Entity, HashMap<String, i32>>,
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
    pub fn add_effect(&mut self, effect: VisualEffect) {
        self.add_effect_by_key(DEFAULT_EFFECT_KEY, effect)
    }
    pub fn add_effects_by_key(&mut self, key: &str, effects: Vec<VisualEffect>) {
        effects
            .into_iter()
            .for_each(|effect| self.add_effect_by_key(key, effect))
    }
    pub fn add_effects(&mut self, effects: Vec<VisualEffect>) {
        self.add_effects_by_key(DEFAULT_EFFECT_KEY, effects)
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
        other
            .active_statuses
            .iter()
            .for_each(|(entity, other_statuses)| {
                let mut statuses = node.active_statuses.remove(entity).unwrap_or_default();
                other_statuses.iter().for_each(|(name, context)| {
                    statuses.insert(name.clone(), context.clone());
                });
                node.active_statuses.insert(*entity, statuses);
            })
    }
    pub fn save_active_statuses(&mut self, pool: &StatusPool) {
        self.active_statuses = pool.active_statuses.clone();
    }
    pub fn save_entity_statuses(&mut self, entity: legion::Entity, pool: &StatusPool) {
        if let Some(statuses) = pool.active_statuses.get(&entity) {
            self.active_statuses.insert(entity, statuses.clone());
        }
    }
    pub fn get_entity_statuses(&self, entity: &legion::Entity) -> Vec<(String, i32)> {
        self.active_statuses
            .get(entity)
            .and_then(|statuses| {
                Some(
                    statuses
                        .iter()
                        .map(|(name, charges)| (name.clone(), *charges))
                        .collect_vec(),
                )
            })
            .unwrap_or_else(|| vec![])
    }
    pub fn finish(mut self, world: &mut legion::World, resources: &Resources) -> Self {
        if self.duration == 0.0 {
            return self;
        }
        let factions = &hashset! {Faction::Light, Faction::Dark, Faction::Shop, Faction::Team};
        ContextSystem::refresh_factions(factions, world, resources);
        UnitSystem::draw_all_units_to_cassette_node(factions, &mut self, world, resources);
        self
    }
}
