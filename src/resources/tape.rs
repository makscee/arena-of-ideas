use super::*;

use geng::prelude::itertools::Itertools;

#[derive(Default)]
pub struct Tape {
    pub persistent_node: Node,                // always rendered
    panels: Vec<(legion::Entity, NodePanel)>, // rendered on top until removed
    cluster_chain: Vec<NodeCluster>,          // for recording
    cluster_queue: ClusterQueue,              // for one time play
}

pub struct NodePanel {
    open: bool,
    pub node: Node,
    ts: Time,
}

#[derive(Default)]
struct ClusterQueue {
    pub clusters: Vec<(Time, NodeCluster)>,
}

#[derive(Default)]
pub struct NodeCluster {
    nodes: Vec<Node>,
    duration: Option<Time>,
    delay_per_node: Option<f32>,
}

#[derive(Default, Clone, Debug)]
pub struct Node {
    entities: HashMap<legion::Entity, Shader>,
    key_effects: HashMap<String, Vec<TimedEffect>>,
    effects: Vec<TimedEffect>,
    duration: Option<Time>,
}

impl Tape {
    pub fn get_shaders(
        &mut self,
        ts: Time,
        mut entity_shaders: HashMap<legion::Entity, Shader>,
        resources: &mut Resources,
    ) -> Vec<Shader> {
        let mut node = self.persistent_node.clone();
        if let Some((start_ts, cluster)) = self.try_get_cluster(ts) {
            let cluster_node = cluster.generate_node(ts - start_ts);
            node.merge(&cluster_node, start_ts, true);
        }
        if let Some((start_ts, queue_node)) = self.cluster_queue.get_node(ts) {
            node.merge_effects(&queue_node, start_ts, true);
            node.merge_entities(&queue_node, false);
        }
        self.panels
            .retain(|(_, panel)| panel.join_node(&mut node, ts));
        entity_shaders.extend(node.get_entity_shaders());

        for effect in node.all_effects() {
            let t = (ts - effect.delay) / effect.duration.unwrap_or(1.0);
            effect.animation.update_entities(t, &mut entity_shaders);
        }
        UnitSystem::inject_entity_shaders_uniforms(&mut entity_shaders, resources);

        let mut extra_shaders: Vec<Shader> = default();
        for effect in node.all_effects() {
            let t = (ts - effect.delay) / effect.duration.unwrap_or(1.0);
            if t > 0.0 {
                extra_shaders.extend(effect.animation.generate_shaders(t, &entity_shaders));
            }
        }
        entity_shaders
            .into_values()
            .chain(extra_shaders.into_iter())
            .collect_vec()
    }

    pub fn push(&mut self, cluster: NodeCluster) {
        self.cluster_chain.push(cluster)
    }

    pub fn push_to_queue(&mut self, cluster: NodeCluster, head: Time) {
        self.cluster_queue.push(cluster, head)
    }

    pub fn length(&self) -> Time {
        self.cluster_chain
            .iter()
            .map(|x| x.get_duration())
            .sum::<Time>()
    }

    fn try_get_cluster(&self, ts: Time) -> Option<(Time, &NodeCluster)> {
        let mut start_ts = 0.0;
        for cluster in self.cluster_chain.iter() {
            let duration = cluster.get_duration();
            if start_ts + duration > ts {
                return Some((start_ts, cluster));
            }
            start_ts += duration;
        }
        None
    }

    pub fn close_panels(&mut self, panel_entity: legion::Entity, ts: Time) -> bool {
        let mut result = false;
        for (entity, panel) in self.panels.iter_mut() {
            if *entity == panel_entity && panel.set_open(false, ts) {
                result = true;
            }
        }
        result
    }

    pub fn close_all_panels(&mut self, ts: Time) {
        for (_, panel) in self.panels.iter_mut() {
            panel.set_open(false, ts);
        }
    }

    pub fn push_panel(&mut self, entity: legion::Entity, panel: NodePanel) {
        self.panels.push((entity, panel));
    }
}

impl NodePanel {
    pub fn new(node: Node, ts: Time) -> Self {
        Self {
            open: true,
            node,
            ts,
        }
    }

    pub fn join_node(&self, node: &mut Node, ts: Time) -> bool {
        let half = self.node.duration() * 0.5;
        let t = match self.open {
            true => {
                if ts > self.ts {
                    (ts - self.ts).min(half)
                } else {
                    return true;
                }
            }
            false => {
                if ts > self.ts + half {
                    return false;
                } else {
                    half + ts - self.ts
                }
            }
        };
        node.merge(&self.node, ts - t, true);
        true
    }

    pub fn set_open(&mut self, value: bool, ts: Time) -> bool {
        if self.open != value {
            self.open = value;
            self.ts = ts;
            true
        } else {
            false
        }
    }
}

impl NodeCluster {
    pub fn new(node: Node) -> Self {
        Self {
            nodes: vec![node],
            ..default()
        }
    }

    /// ts: 0.0 -> duration
    pub fn generate_node(&self, ts: Time) -> Node {
        let mut result: Node = default();
        let mut cur_ts = 0.0;
        for node in self.nodes.iter() {
            if cur_ts > ts {
                break;
            }
            let node_duration = node.duration();
            if cur_ts + node_duration > ts || node_duration == 0.0 {
                result.merge_effects(node, cur_ts, true);
            }
            result.merge_entities(node, true);
            cur_ts += match self.delay_per_node {
                Some(value) => value,
                None => node_duration,
            }
        }
        result
    }

    pub fn set_duration(&mut self, duration: Time) {
        self.duration = Some(duration);
        if duration > self.nodes_duration() {
            return;
        }
        let mut cur_ts = 0.0;
        let per_node = duration / self.nodes.len() as f32 * 0.75;
        self.delay_per_node = Some(per_node);
        for node in self.nodes.iter_mut() {
            if node.duration() + cur_ts > duration {
                node.set_max_duration(duration - cur_ts);
            }
            cur_ts += per_node;
        }
    }

    pub fn get_duration(&self) -> Time {
        self.duration.unwrap_or(self.nodes_duration())
    }

    pub fn push(&mut self, node: Node) {
        self.nodes.push(node)
    }

    pub fn push_front(&mut self, node: Node) {
        self.nodes.insert(0, node)
    }

    fn nodes_duration(&self) -> Time {
        self.nodes.iter().map(|x| x.duration()).sum::<Time>()
    }
}

impl ClusterQueue {
    pub fn push(&mut self, cluster: NodeCluster, head: Time) {
        self.clusters.push((head, cluster));
    }

    pub fn get_node(&mut self, head: Time) -> Option<(Time, Node)> {
        let mut node = Node::default();
        let mut start_ts: Option<Time> = None;
        self.clusters.retain(|(cluster_start, cluster)| {
            let duration = cluster.get_duration();
            if cluster_start + duration < head {
                return false;
            }
            if start_ts.is_none() {
                start_ts = Some(*cluster_start);
            }
            node.merge(
                &cluster.generate_node(head - cluster_start),
                cluster_start - start_ts.unwrap(),
                true,
            );
            true
        });
        start_ts.and_then(|x| Some((x, node)))
    }
}

impl Node {
    pub fn new_panel_scaled(shader: Shader) -> Node {
        let animation = AnimatedShaderUniforms::empty()
            .add_key_frame(
                0.0,
                hashmap! {"u_open"=> ShaderUniform::Float(0.0)}.into(),
                EasingType::Linear,
            )
            .add_key_frame(
                0.5,
                hashmap! {"u_open"=> ShaderUniform::Float(1.0)}.into(),
                EasingType::QuadOut,
            )
            .add_key_frame(
                1.0,
                hashmap! {"u_open"=> ShaderUniform::Float(0.0)}.into(),
                EasingType::QuadIn,
            );
        let animation = Animation::ShaderAnimation { shader, animation };
        let mut node = Node::default();
        node.add_effect(TimedEffect::new(Some(1.0), animation, 0));
        node
    }

    pub fn add_effect_by_key(&mut self, key: String, effect: TimedEffect) {
        self.add_effects_by_key(key, vec![effect])
    }

    pub fn add_effects_by_key(&mut self, key: String, effects: Vec<TimedEffect>) {
        self.key_effects.entry(key).or_default().extend(effects)
    }

    pub fn add_effect(&mut self, effect: TimedEffect) {
        self.effects.push(effect);
    }

    pub fn add_effects(&mut self, effects: Vec<TimedEffect>) {
        self.effects.extend(effects)
    }

    pub fn merge(&mut self, other: &Node, add_delay: Time, force: bool) -> &mut Self {
        self.merge_effects(other, add_delay, force);
        self.merge_entities(other, force);
        self
    }

    pub fn merge_entities(&mut self, other: &Node, force: bool) -> &mut Self {
        for (entity, data) in other.entities.iter() {
            if force || !self.entities.contains_key(entity) {
                self.entities.insert(*entity, data.clone());
            }
        }
        self
    }

    pub fn merge_effects(&mut self, other: &Node, add_delay: Time, force: bool) -> &mut Self {
        self.effects
            .extend(other.effects.iter().cloned().map(|mut x| {
                x.delay += add_delay;
                x
            }));
        for (key, effects) in other.key_effects.iter() {
            if force || !self.key_effects.contains_key(key) {
                self.key_effects.insert(
                    key.clone(),
                    effects
                        .iter()
                        .cloned()
                        .map(|mut x| {
                            x.delay += add_delay;
                            x
                        })
                        .collect_vec(),
                );
            }
        }
        self
    }

    pub fn add_entity_shader(&mut self, entity: legion::Entity, shader: Shader) {
        if let Some(data) = self.entities.get_mut(&entity) {
            *data = shader;
        } else {
            self.entities.insert(entity, shader);
        }
    }

    pub fn get_entity_shaders(&self) -> HashMap<legion::Entity, Shader> {
        HashMap::from_iter(
            self.entities
                .iter()
                .map(|(entity, shader)| (*entity, shader.clone())),
        )
    }

    pub fn key_effects_count(&self, key: &str) -> usize {
        match self.key_effects.get(key) {
            Some(v) => v.len(),
            None => 0,
        }
    }

    pub fn all_effects(&self) -> impl Iterator<Item = &TimedEffect> {
        self.effects
            .iter()
            .chain(self.key_effects.values().flatten())
    }

    pub fn duration(&self) -> Time {
        self.duration.expect("Node is not locked")
    }

    pub fn set_max_duration(&mut self, duration: Time) {
        let old_duration = self.duration();
        if old_duration <= duration {
            return;
        }
        let mul = duration / old_duration;
        for effect in self
            .effects
            .iter_mut()
            .chain(self.key_effects.values_mut().flatten())
        {
            effect.delay *= mul;
            if let Some(duration) = effect.duration.as_mut() {
                *duration *= mul;
            }
        }
        self.duration = Some(duration);
    }

    pub fn lock(mut self, lock_type: NodeLockType) -> Self {
        self.duration = Some(
            self.all_effects()
                .map(|x| x.duration.unwrap_or_default() + x.delay)
                .reduce(|a, b| a.max(b))
                .unwrap_or_default(),
        );
        let lock_type = match lock_type {
            NodeLockType::Full { world, resources } => NodeLockType::Factions {
                factions: HashSet::from_iter(Faction::all_iter()),
                world,
                resources,
            },
            _ => lock_type,
        };
        match lock_type {
            NodeLockType::Factions {
                world,
                resources,
                factions,
            } => {
                UnitSystem::draw_all_units_to_node(&factions, &mut self, world, resources);
                SlotSystem::refresh_slots(factions, world, resources);
                Self::draw_all_tape_entities_to_node(&mut self, world);
            }
            NodeLockType::Empty => {}
            _ => panic!("Wrong lock type"),
        }
        self
    }

    pub fn push_as_panel(self, entity: legion::Entity, resources: &mut Resources) {
        let panel = NodePanel::new(self, resources.tape_player.head);
        resources.tape_player.tape.push_panel(entity, panel);
    }

    fn draw_all_tape_entities_to_node(node: &mut Node, world: &legion::World) {
        <(&EntityComponent, &Shader)>::query()
            .filter(component::<TapeEntityComponent>())
            .iter(world)
            .for_each(|(entity, shader)| {
                node.add_entity_shader(entity.entity, shader.clone());
            })
    }
}

pub enum NodeLockType<'a> {
    Full {
        world: &'a mut legion::World,
        resources: &'a mut Resources,
    },
    Factions {
        factions: HashSet<Faction>,
        world: &'a mut legion::World,
        resources: &'a mut Resources,
    },
    Empty,
}
