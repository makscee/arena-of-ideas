use super::*;

mod condition;

#[derive(Clone)]
pub struct QueuedEffect<T> {
    pub effect: T,
    pub context: EffectContext,
}

pub struct EffectOrchestrator {
    effects: HashMap<String, VecDeque<QueuedEffect<Effect>>>,
    delays: HashMap<String, f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EffectContext {
    pub queue_id: Option<String>,
    pub owner: Id,
    pub creator: Id,
    pub target: Id,
    pub vars: HashMap<VarName, i32>,
    pub status_id: Option<Id>,
    pub color: Rgba<f32>,
}

impl EffectOrchestrator {
    pub fn new() -> Self {
        Self {
            effects: hashmap! {},
            delays: hashmap! {},
        }
    }

    pub fn add_delay(&mut self, context: &EffectContext, value: f32) {
        let q_id = context.get_q_id();
        self.add_delay_by_id(q_id, value);
    }

    pub fn add_delay_by_id(&mut self, q_id: String, value: f32) {
        if value <= 0.0 {
            return;
        }
        let mut new_value = *self.delays.get(&q_id).unwrap_or(&0.0);
        new_value = value.max(new_value);
        self.delays.insert(q_id, new_value);
    }

    pub fn update_delays(&mut self, delta_time: f32) {
        for v in self.delays.values_mut() {
            *v = *v - delta_time;
        }
    }

    fn check_queue(&mut self, q_id: &String) {
        if !self.delays.contains_key(q_id) {
            self.delays.insert(q_id.clone(), 0.0);
        }
        if !self.effects.contains_key(q_id) {
            self.effects.insert(q_id.clone(), VecDeque::new());
        }
    }

    pub fn push_front(&mut self, context: EffectContext, effect: Effect) {
        let q_id = context.get_q_id();
        self.check_queue(&q_id);
        self.effects
            .get_mut(&q_id)
            .unwrap()
            .push_front(QueuedEffect {
                effect,
                context: context.clone(),
            });
    }

    pub fn push_back(&mut self, context: EffectContext, effect: Effect) {
        let q_id = context.get_q_id();
        self.check_queue(&q_id);
        self.effects
            .get_mut(&q_id)
            .unwrap()
            .push_back(QueuedEffect {
                effect,
                context: context.clone(),
            });
    }

    fn is_queue_delayed(&self, q_id: &String) -> bool {
        self.delays
            .get(q_id)
            .and_then(|x| Some(*x > 0.0))
            .or(Some(false))
            .unwrap()
    }

    fn get_available_q_id(&self) -> Option<String> {
        for q_id in self.effects.keys() {
            if self.is_queue_delayed(q_id) {
                continue;
            }
            if !self.effects[q_id].is_empty() {
                return Some(q_id.clone());
            }
        }
        None
    }

    pub fn collect_next_effects(&mut self) -> Vec<QueuedEffect<Effect>> {
        let mut result = vec![];
        let qs = self.effects.keys().map(|x| x.clone()).collect_vec();
        for q_id in qs {
            if self.is_queue_delayed(&q_id) || self.effects[&q_id].is_empty() {
                continue;
            }
            result.push(self.effects.get_mut(&q_id).unwrap().pop_front().unwrap());
        }
        result
    }

    pub fn clear(&mut self) {
        self.effects.clear();
        self.delays.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.delays.values().all(|x| *x <= 0.0) && self.effects.values().all(|x| x.is_empty())
    }
}

impl EffectContext {
    pub fn get_q_id(&self) -> String {
        if let Some(queue_id) = &self.queue_id {
            queue_id.clone()
        } else {
            self.owner.to_string()
        }
    }
    pub fn get_id(&self, who: Who) -> Id {
        match who {
            Who::Owner => self.owner,
            Who::Creator => self.creator,
            Who::Target => self.target,
        }
    }
    pub fn to_string(&self, logic: &Logic) -> String {
        format!(
            "owner: {}, creator: {}, target: {} status: {}",
            self.unit_to_string(Some(self.owner), logic),
            self.unit_to_string(Some(self.creator), logic),
            self.unit_to_string(Some(self.target), logic),
            match self.status_id {
                None => "None".to_string(),
                Some(id) => id.to_string(),
            }
        )
    }
    pub fn unit_to_string(&self, unit: Option<Id>, logic: &Logic) -> String {
        match unit {
            Some(id) => {
                if let Some(unit) = logic.model.units.get(&id) {
                    format!("{}#{}", unit.unit_type, id)
                } else {
                    let unit = logic.model.dead_units.get(&id);
                    if let Some(unit) = unit {
                        format!("{}#{}(dead)", unit.unit_type, id)
                    } else {
                        format!("{}(not found)", id)
                    }
                }
            }
            None => "None".to_owned(),
        }
    }
}

impl Logic {
    pub fn process_effects(&mut self) {
        const MAX_ITERATIONS: usize = 1000;
        let mut iterations = 0;
        let mut next_effects = self.effects.collect_next_effects();
        while !next_effects.is_empty() {
            let QueuedEffect {
                effect,
                mut context,
            } = next_effects.pop().unwrap();
            self.model.vars.iter().for_each(|v| {
                if !context.vars.contains_key(v.0) {
                    context.vars.insert(v.0.clone(), *v.1);
                }
            });
            debug!(
                "Processing q#{} {:?} on {}",
                context.get_q_id(),
                effect,
                context.to_string(self)
            );
            effect.as_box().process(context, self);
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                error!("Exceeded effect processing limit: {}", MAX_ITERATIONS);
                break;
            }
            if next_effects.is_empty() {
                next_effects = self.effects.collect_next_effects();
            }
        }
    }
    pub fn process_delays(&mut self, delta_time: f32) {
        self.effects.update_delays(delta_time);
    }
}
