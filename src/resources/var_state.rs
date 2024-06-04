use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct VarState {
    vars: HashMap<VarName, HashMap<String, History>>,
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
}

impl History {
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
