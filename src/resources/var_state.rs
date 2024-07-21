use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct VarState {
    vars: HashMap<VarName, HashMap<String, History>>,
    statuses: HashMap<String, VarState>,
    birth: f32,
    id: GID,
    entity: Option<Entity>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
struct History(Vec<VarChange>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct VarChange {
    pub t: f32,
    #[serde(default)]
    pub duration: f32,
    #[serde(default)]
    pub timeframe: f32,
    #[serde(default)]
    pub tween: Tween,
    pub value: VarValue,
}

impl VarChange {
    pub fn new(value: VarValue) -> Self {
        Self {
            t: default(),
            duration: default(),
            timeframe: default(),
            tween: default(),
            value,
        }
    }
}

impl VarState {
    pub fn birth(&self) -> f32 {
        self.birth
    }
    pub fn entity(&self) -> Option<Entity> {
        self.entity
    }
    pub fn new_with(var: VarName, value: VarValue) -> Self {
        Self::default().init(var, value).take()
    }
    pub fn id(&self) -> GID {
        self.id
    }
    pub fn attach(mut self, entity: Entity, id: GID, world: &mut World) {
        self.birth = gt().insert_head();
        self.id = id;
        self.entity = Some(entity);
        world.entity_mut(entity).insert(self);
    }
    pub fn get(entity: Entity, world: &World) -> &Self {
        Self::try_get(entity, world).unwrap()
    }
    pub fn try_get(entity: Entity, world: &World) -> Result<&Self> {
        world
            .get::<Self>(entity)
            .or_else(|| {
                world.get::<Status>(entity).and_then(|s| {
                    entity.get_parent(world).and_then(|p| {
                        Self::try_get(p, world)
                            .ok()
                            .and_then(|state| state.get_status(&s.name))
                    })
                })
            })
            .with_context(|| format!("VarState not found for {entity:?}"))
    }
    pub fn get_mut(entity: Entity, world: &mut World) -> Mut<Self> {
        Self::try_get_mut(entity, world).unwrap()
    }
    pub fn try_get_mut(entity: Entity, world: &mut World) -> Result<Mut<Self>> {
        world
            .get_mut::<Self>(entity)
            .with_context(|| format!("VarState not found for {entity:?}"))
    }
    pub fn add_status(&mut self, name: String, state: VarState) {
        self.statuses.insert(name, state);
    }
    pub fn get_status(&self, name: &str) -> Option<&VarState> {
        self.statuses.get(name)
    }
    pub fn get_status_mut(&mut self, name: &str) -> Option<&mut VarState> {
        self.statuses.get_mut(name)
    }
    pub fn reindex_statuses(&mut self) {
        let mut i = 0;
        for state in self
            .statuses
            .values_mut()
            .sorted_by(|a, b| a.birth.total_cmp(&b.birth))
        {
            if state
                .get_value_last(VarName::Visible)
                .and_then(|v| v.get_bool())
                .unwrap_or(true)
            {
                state.set_int(VarName::StatusIndex, i.into());
                i += 1;
            }
        }
    }
    pub fn all_statuses_at(&self, t: f32) -> HashMap<String, i32> {
        HashMap::from_iter(self.statuses.iter().filter_map(|(name, state)| {
            if LOCAL_STATUS.eq(name) {
                None
            } else {
                Some((
                    name.into(),
                    state
                        .get_value_at(VarName::Charges, t)
                        .and_then(|v| v.get_int())
                        .unwrap_or_default(),
                ))
            }
        }))
    }
    pub fn init(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.vars.insert(
            var,
            HashMap::from([(String::default(), History::new(value))]),
        );
        self
    }
    pub fn push_change(&mut self, var: VarName, key: String, mut change: VarChange) -> &mut Self {
        let head = gt().insert_head();
        let birth = self.birth;
        change.t += head - birth;
        self.vars
            .entry(var)
            .or_insert(default())
            .entry(key)
            .or_insert(default())
            .push_change(change);
        self
    }
    pub fn has_value(&self, var: VarName) -> bool {
        self.vars.contains_key(&var)
    }
    pub fn get_value_at(&self, var: VarName, t: f32) -> Result<VarValue> {
        self.vars
            .get(&var)
            .with_context(|| format!("Var {var} not set for {:?}", self.entity))?
            .iter()
            .filter_map(|(_, v)| v.get_value_at(t - self.birth).ok())
            .reduce(|acc, v| match VarValue::sum(&acc, &v) {
                Ok(v) => v,
                Err(_) => acc,
            })
            .context("Value not found")
    }
    pub fn get_value_last(&self, var: VarName) -> Result<VarValue> {
        Ok(self
            .vars
            .get(&var)
            .with_context(|| format!("Var {var} not set for {:?}", self.entity))?
            .iter()
            .filter_map(|(_, v)| v.get_value_last())
            .reduce(|acc, v| match VarValue::sum(&acc, &v) {
                Ok(v) => v,
                Err(_) => acc,
            })
            .unwrap_or_default())
    }
    pub fn get_key_value_last(&self, key: &str, var: VarName) -> Result<VarValue> {
        Ok(self
            .vars
            .get(&var)
            .with_context(|| format!("Var {var} not set for {:?}", self.entity))?
            .get(key)
            .and_then(|h| h.get_value_last())
            .unwrap_or_default())
    }
    pub fn get_key_value_at(&self, key: &str, var: VarName, t: f32) -> Result<VarValue> {
        Ok(self
            .vars
            .get(&var)
            .with_context(|| format!("Var {var} not set for {:?}", self.entity))?
            .get(key)
            .and_then(|h| h.get_value_at(t - self.birth).ok())
            .unwrap_or_default())
    }

    pub fn set_value(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.push_change(var, default(), VarChange::new(value));
        self
    }
    pub fn set_key_value(&mut self, key: String, var: VarName, value: VarValue) -> &mut Self {
        self.push_change(var, key, VarChange::new(value));
        self
    }

    pub fn set_int(&mut self, var: VarName, value: i32) -> &mut Self {
        self.push_change(var, default(), VarChange::new(VarValue::Int(value)));
        self
    }
    pub fn change_int(&mut self, var: VarName, delta: i32) -> i32 {
        let value = self
            .get_value_last(var)
            .and_then(|v| v.get_int())
            .unwrap_or_default()
            + delta;
        self.set_int(var, value);
        value
    }
    pub fn set_vec2(&mut self, var: VarName, value: Vec2) -> &mut Self {
        self.push_change(var, default(), VarChange::new(VarValue::Vec2(value)));
        self
    }
    pub fn change_vec2(&mut self, var: VarName, delta: Vec2) -> Vec2 {
        let value = self
            .get_value_last(var)
            .and_then(|v| v.get_vec2())
            .unwrap_or_default()
            + delta;
        self.set_vec2(var, value);
        delta
    }
    pub fn set_color(&mut self, var: VarName, value: Color) -> &mut Self {
        self.push_change(var, default(), VarChange::new(VarValue::Color(value)));
        self
    }

    pub fn all_values(&self, t: f32) -> HashMap<VarName, VarValue> {
        HashMap::from_iter(
            self.vars
                .keys()
                .filter_map(|k| match self.get_value_at(*k, t) {
                    Ok(v) => Some((*k, v)),
                    Err(_) => None,
                }),
        )
    }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl History {
    fn new(value: VarValue) -> Self {
        Self(vec![VarChange::new(value)])
    }
    fn get_value_at(&self, t: f32) -> Result<VarValue> {
        if t < 0.0 {
            return Err(anyhow!("Not born yet"));
        }
        if self.0.is_empty() {
            return Err(anyhow!("History is empty"));
        }

        let i = self.0.partition_point(|x| x.t <= t);
        if i == 0 {
            return Err(anyhow!("First change not reached {t}"));
        }
        let cur_change = &self.0[i - 1];
        let prev_change = if i > 1 { &self.0[i - 2] } else { cur_change };
        let t = t - cur_change.t;
        cur_change.tween.f(
            &prev_change.value,
            &cur_change.value,
            t,
            cur_change.duration,
        )
    }
    fn get_value_last(&self) -> Option<VarValue> {
        self.0.last().map(|v| v.value.clone())
    }
    fn push_change(&mut self, change: VarChange) {
        if let Some(last) = self.0.last() {
            if change.value.eq(&last.value) {
                return;
            }
            if change.duration == 0.0 && last.t == change.t {
                self.0.remove(self.0.len() - 1);
            }
        }
        self.0.push(change);
    }
}
