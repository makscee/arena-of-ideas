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
        if last.end < head {
            last.end = head;
        }
        nodes.drain(..).for_each(|mut node| {
            let last = self.last();
            let start = last.start + (last.end - last.start) * (1.0 - last.skip_part);
            node.start = start;
            node.end = node.start + node.end;
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
        mut entity_shaders: HashMap<legion::Entity, Shader>,
    ) -> Vec<Shader> {
        let cassette = &resources.cassette;
        let mut node = cassette.render_node.clone();
        let time = cassette.head - node.start;
        Self::get_nodes_at_ts(cassette, cassette.head)
            .into_iter()
            .for_each(|x| {
                node.merge_mut(x, true);
            });
        entity_shaders.extend(node.get_entities_shaders());

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
        node.add_effects(StatusSystem::get_active_statuses_panel_effects(
            &node, resources,
        ));

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
        last.end
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

    fn get_nodes_at_ts(&self, ts: Time) -> Vec<&CassetteNode> {
        let mut nodes: Vec<&CassetteNode> = default();
        for node in self.tape.iter() {
            if node.start > ts {
                break;
            }
            if node.end > ts {
                nodes.push(node);
            }
        }
        nodes
    }
}

#[derive(Default, Clone, Debug)]

pub struct CassetteNode {
    pub start: Time,
    pub end: Time,
    pub skip_part: f32,
    entities: HashMap<legion::Entity, EntityData>,
    effects: HashMap<String, Vec<VisualEffect>>,
}

#[derive(Clone, Debug)]
struct EntityData {
    pub shader: Shader,
    pub statuses: HashMap<String, i32>,
    pub definitions: HashSet<String>,
}

impl EntityData {
    fn new(shader: Shader) -> Self {
        Self {
            shader,
            statuses: default(),
            definitions: default(),
        }
    }
}

impl CassetteNode {
    pub fn add_entity_shader(&mut self, entity: legion::Entity, shader: Shader) {
        if let Some(data) = self.entities.get_mut(&entity) {
            data.shader = shader;
        } else {
            self.entities.insert(entity, EntityData::new(shader));
        }
    }
    pub fn get_entities_shaders(&self) -> HashMap<legion::Entity, Shader> {
        HashMap::from_iter(
            self.entities
                .iter()
                .map(|(entity, data)| (*entity, data.shader.clone())),
        )
    }
    pub fn get_active_statuses(&self, entity: legion::Entity) -> HashMap<String, i32> {
        if let Some(data) = self.entities.get(&entity) {
            data.statuses.clone()
        } else {
            default()
        }
    }
    pub fn get_definitions(&self, entity: legion::Entity) -> Vec<&String> {
        if let Some(data) = self.entities.get(&entity) {
            data.statuses
                .keys()
                .chain(data.definitions.iter())
                .collect_vec()
        } else {
            default()
        }
    }
    pub fn add_effect_by_key(&mut self, key: &str, effect: VisualEffect) {
        self.end = self.end.max(self.start + effect.duration + effect.delay);
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
        self.end = default();
        self.entities.clear();
        self.effects.clear();
    }
    pub fn clear_entities(&mut self) {
        self.entities.clear();
    }
    pub fn merge(&self, other: &CassetteNode, force: bool) -> CassetteNode {
        let mut node = self.clone();
        node.merge_mut(other, force);
        node
    }
    pub fn merge_mut(&mut self, other: &CassetteNode, force: bool) {
        let mut node = self;
        let add_delay = other.start - node.start;
        for (key, other_effects) in other.effects.iter() {
            if key == DEFAULT_EFFECT_KEY {
                let mut effects = node.effects.remove(key).unwrap_or_default();
                for mut effect in other_effects.iter().cloned() {
                    effect.delay += add_delay;
                    effects.push(effect);
                }
                node.effects.insert(key.clone(), effects);
            } else {
                if force || !node.effects.contains_key(key) {
                    node.effects.insert(
                        key.clone(),
                        other_effects
                            .iter()
                            .map(|x| {
                                let mut x = x.clone();
                                x.delay += add_delay;
                                x
                            })
                            .collect_vec(),
                    );
                }
            }
        }
        node.start = node.start.min(other.start);
        node.end = node.end.max(other.end);
        other.entities.iter().for_each(|(entity, shader)| {
            if force || !node.entities.contains_key(entity) {
                node.entities.insert(*entity, shader.clone());
            }
        });
    }
    pub fn save_entity_statuses(&mut self, entity: legion::Entity, pool: &StatusPool) {
        if let Some(statuses) = pool.active_statuses.get(&entity) {
            self.entities.get_mut(&entity).unwrap().statuses = statuses.clone();
        }
    }
    pub fn save_entity_definitions(
        &mut self,
        entity: legion::Entity,
        definitions: HashSet<String>,
    ) {
        self.entities.get_mut(&entity).unwrap().definitions = definitions;
    }
    pub fn finish(mut self, world: &mut legion::World, resources: &Resources) -> Self {
        if self.end == 0.0 {
            return self;
        }
        let factions = &hashset! {Faction::Light, Faction::Dark, Faction::Shop, Faction::Team};
        ContextSystem::refresh_factions(factions, world, resources);
        UnitSystem::draw_all_units_to_cassette_node(factions, &mut self, world, resources);
        self.add_effects_by_key(TEAM_NAMES_KEY, VfxSystem::vfx_battle_team_names(resources));
        self
    }
}
