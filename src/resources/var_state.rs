use bevy::math::Quat;

use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct VarState {
    vars: HashMap<VarName, HashMap<String, History>>,
    statuses: HashMap<String, VarState>,
    birth: f32,
    id: u64,
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
    pub fn new_with(var: VarName, value: VarValue) -> Self {
        Self::default().init(var, value).take()
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn attach(mut self, entity: Entity, id: u64, world: &mut World) {
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
        let head = gt().insert_head();
        let birth = self.birth;
        change.t += head - birth;
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
            .filter_map(|(_, v)| v.get_value_at(t - self.birth).ok())
            .reduce(|acc, v| match VarValue::sum(&acc, &v) {
                Ok(v) => v,
                Err(_) => acc,
            })
            .unwrap_or_default())
    }
    pub fn find_value_at(entity: Entity, var: VarName, t: f32, world: &World) -> Result<VarValue> {
        match Self::try_get(entity, world).and_then(|s| s.get_value_at(var, t)) {
            Ok(v) => Ok(v),
            Err(_) => Self::find_value_at(
                entity.get_parent(world).context("No parent")?,
                var,
                t,
                world,
            ),
        }
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

    pub fn get_vec2(&self, var: VarName) -> Result<Vec2> {
        self.get_value_last(var)?.get_vec2()
    }
    pub fn set_vec2(&mut self, var: VarName, value: Vec2) -> &mut Self {
        self.push_change(var, default(), VarChange::new(VarValue::Vec2(value)));
        self
    }
    pub fn change_vec2(&mut self, var: VarName, delta: Vec2) -> Vec2 {
        let value = self.get_vec2(var).unwrap_or_default() + delta;
        self.set_vec2(var, value);
        delta
    }

    pub fn get_bool_at(&self, var: VarName, t: f32) -> Result<bool> {
        self.get_value_at(var, t)?.get_bool()
    }

    pub fn get_string(&self, var: VarName) -> Result<String> {
        self.get_value_last(var)?.get_string()
    }
    pub fn get_string_at(&self, var: VarName, t: f32) -> Result<String> {
        self.get_value_at(var, t)?.get_string()
    }

    pub fn get_faction(&self, var: VarName) -> Result<Faction> {
        self.get_value_last(var)?.get_faction()
    }

    pub fn all_values(&self) -> HashMap<VarName, VarValue> {
        HashMap::from_iter(
            self.vars
                .keys()
                .filter_map(|k| match self.get_value_last(*k) {
                    Ok(v) => Some((*k, v)),
                    Err(_) => None,
                }),
        )
    }

    pub fn apply_transform(entity: Entity, t: f32, vars: Vec<VarName>, world: &mut World) {
        let mut e = world.entity_mut(entity);
        let mut transform = e.get::<Transform>().unwrap().clone();
        let state = e.get::<VarState>().unwrap();
        for var in vars {
            match var {
                VarName::Position => {
                    let position = state
                        .get_value_at(var, t)
                        .and_then(|x| x.get_vec2())
                        .unwrap_or_default();
                    transform.translation.x = position.x;
                    transform.translation.y = position.y;
                }
                VarName::Scale => {
                    let scale = state
                        .get_value_at(var, t)
                        .and_then(|x| x.get_vec2())
                        .unwrap_or(Vec2::ONE);
                    transform.scale.x = scale.x;
                    transform.scale.y = scale.y;
                }
                VarName::Rotation => {
                    let rotation = state
                        .get_value_at(var, t)
                        .and_then(|x| x.get_float())
                        .unwrap_or_default();
                    transform.rotation = Quat::from_rotation_z(rotation);
                }
                VarName::Offset => {
                    let position = state
                        .get_value_at(var, t)
                        .and_then(|x| x.get_vec2())
                        .unwrap_or_default();
                    transform.translation.x = position.x;
                    transform.translation.y = position.y;
                }
                _ => {}
            }
        }
        e.insert(transform);
    }

    pub fn sort_history(mut self) -> Self {
        self.vars.values_mut().for_each(|v| {
            v.values_mut().for_each(|h| {
                h.0.sort_by(|a, b| a.t.total_cmp(&b.t));
            })
        });
        self
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
