use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct VarState {
    vars: HashMap<VarName, HashMap<String, History>>,
    statuses: HashMap<String, VarState>,
    birth: f32,
    entity: Option<Entity>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
struct History(Vec<VarChange>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    pub fn attach(mut self, entity: Entity, world: &mut World) {
        self.birth = GameTimer::get().insert_head();
        self.entity = Some(entity);
        world.entity_mut(entity).insert(self);
    }
    pub fn get(entity: Entity, world: &World) -> &Self {
        Self::try_get(entity, world).unwrap()
    }
    pub fn try_get(entity: Entity, world: &World) -> Result<&Self> {
        world
            .get::<Self>(entity)
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
    pub fn init(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.vars.insert(
            var,
            HashMap::from([(String::default(), History::new(value))]),
        );
        self
    }
    pub fn push_change(&mut self, var: VarName, key: String, mut change: VarChange) -> &mut Self {
        let head = GameTimer::get().insert_head();
        let birth = self.birth;
        change.t += head - birth;
        GameTimer::get().advance_insert(change.timeframe);
        self.vars
            .entry(var)
            .or_insert(default())
            .entry(key)
            .or_insert(default())
            .0
            .push(change);
        self
    }
    pub fn has_value(&self, var: VarName) -> bool {
        self.vars.contains_key(&var)
    }
    pub fn get_value_at(&self, var: VarName, t: f32) -> Result<VarValue> {
        Ok(self
            .vars
            .get(&var)
            .with_context(|| format!("Var {var} not set for {:?}", self.entity))?
            .iter()
            .filter_map(|(_, v)| v.get_value_at(t).ok())
            .reduce(|acc, v| match VarValue::sum(&acc, &v) {
                Ok(v) => v,
                Err(_) => acc,
            })
            .unwrap_or_default())
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

    pub fn get_int(&self, var: VarName) -> Result<i32> {
        self.get_value_last(var)?.get_int()
    }
    pub fn set_int(&mut self, var: VarName, value: i32) -> &mut Self {
        self.push_change(var, default(), VarChange::new(VarValue::Int(value)));
        self
    }
    pub fn change_int(&mut self, var: VarName, delta: i32) -> i32 {
        let value = self.get_int(var).unwrap_or_default() + delta;
        self.set_int(var, value);
        delta
    }

    pub fn get_faction(&self, var: VarName) -> Result<Faction> {
        self.get_value_last(var)?.get_faction()
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
}
