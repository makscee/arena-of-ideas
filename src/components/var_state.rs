use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, Reflect, Default)]
pub struct VarState {
    pub history: HashMap<VarName, History>,
    #[serde(default)]
    pub birth: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Reflect)]
pub struct History(pub Vec<Change>);

#[derive(Serialize, Deserialize, Clone, Debug, Reflect)]
pub struct Change {
    pub t: f32,
    #[serde(default)]
    pub duration: f32, // over what period the change will be applied
    #[serde(default)]
    pub tween: Tween,
    pub value: VarValue,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Reflect)]
pub enum Tween {
    #[default]
    Linear,
    QuartOut,
    QuartIn,
    QuartInOut,
    QuadOut,
    QuadIn,
    QuadInOut,
    CubicIn,
    CubicOut,
    BackIn,
}

impl VarState {
    pub fn new_with(var: VarName, value: VarValue) -> Self {
        mem::take(Self::default().init(var, value))
    }

    pub fn attach(mut self, entity: Entity, world: &mut World) {
        self.birth = get_insert_t(world);
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
    pub fn find(entity: Entity, world: &World) -> &Self {
        Self::try_find(entity, world).unwrap()
    }
    pub fn try_find(mut entity: Entity, world: &World) -> Result<&Self> {
        loop {
            let state = Self::try_get(entity, world);
            if state.is_ok() {
                return state;
            }
            if let Some(parent) = world.get::<Parent>(entity) {
                entity = parent.get();
            }
        }
    }
    pub fn get_mut(entity: Entity, world: &mut World) -> Mut<Self> {
        Self::try_get_mut(entity, world).unwrap()
    }
    pub fn try_get_mut(entity: Entity, world: &mut World) -> Result<Mut<Self>> {
        world
            .get_mut::<Self>(entity)
            .with_context(|| format!("VarState not found for {entity:?}"))
    }

    pub fn change_int(entity: Entity, var: VarName, delta: i32, world: &mut World) -> Result<()> {
        let value = Self::get(entity, world).get_int(var).unwrap_or_default() + delta;
        Self::push_back(entity, var, Change::new(VarValue::Int(value)), world);
        Ok(())
    }

    pub fn push_back(entity: Entity, var: VarName, mut change: Change, world: &mut World) {
        let end = get_insert_t(world);
        let birth = Self::get(entity, world).birth;
        change.t += end - birth;
        world
            .get_resource_mut::<GameTimer>()
            .unwrap()
            .register_insert(change.total_duration() + birth);
        Self::get_mut(entity, world)
            .history
            .entry(var)
            .or_insert(default())
            .push(change);
    }
    pub fn insert_simple(&mut self, var: VarName, value: VarValue, t: f32) -> &mut Self {
        self.history
            .entry(var)
            .or_insert(default())
            .push(Change::new(value).set_t(t - self.birth));
        self
    }
    pub fn init(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.history.insert(var, History::new(value));
        self
    }
    pub fn get_value_at(&self, var: VarName, t: f32) -> Result<VarValue> {
        self.history
            .get(&var)
            .context("No key in state")?
            .find_value(t - self.birth)
    }
    pub fn get_value_last(&self, var: VarName) -> Result<VarValue> {
        self.history
            .get(&var)
            .with_context(|| format!("Var not found {var}"))?
            .get_last()
            .context("History is empty")
    }
    pub fn get_int(&self, var: VarName) -> Result<i32> {
        self.get_value_last(var)?.get_int()
    }
    pub fn get_faction(&self, var: VarName) -> Result<Faction> {
        self.get_value_last(var)?.get_faction()
    }
    pub fn get_entity(&self, var: VarName) -> Result<Entity> {
        self.get_value_last(var)?.get_entity()
    }
    pub fn get_vec2(&self, var: VarName) -> Result<Vec2> {
        self.get_value_last(var)?.get_vec2()
    }
    pub fn get_bool(&self, var: VarName) -> Result<bool> {
        self.get_value_last(var)?.get_bool()
    }
    pub fn get_bool_at(&self, var: VarName, t: f32) -> Result<bool> {
        self.get_value_at(var, t)?.get_bool()
    }
    pub fn get_string(&self, var: VarName) -> Result<String> {
        self.get_value_last(var)?.get_string()
    }
    pub fn get_color(&self, var: VarName) -> Result<Color> {
        self.get_value_last(var)?.get_color()
    }
    pub fn find_value(mut entity: Entity, var: VarName, t: f32, world: &World) -> Result<VarValue> {
        let mut result = None;
        loop {
            if let Some(state) = world.get::<VarState>(entity) {
                if let Ok(mut value) = state.get_value_at(var, t) {
                    if let Some(children) = world.get::<Children>(entity) {
                        for child in children.to_vec() {
                            if let Some(delta) = world.get::<VarStateDelta>(child) {
                                value = delta.process(var, value, t);
                            }
                        }
                    }
                    result = Some(value);
                    break;
                }
            }
            if let Some(parent) = world.get::<Parent>(entity) {
                entity = parent.get();
                continue;
            }
            break;
        }
        result.with_context(|| format!("Var {var} was not found"))
    }
    pub fn get_value(entity: Entity, var: VarName, t: f32, world: &World) -> Result<VarValue> {
        if let Ok(state) = Self::try_get(entity, world) {
            state.get_value_at(var, t)
        } else {
            Err(anyhow!("Var {var} was not found"))
        }
    }
    pub fn duration(&self) -> f32 {
        self.history
            .values()
            .map(|x| x.duration())
            .max_by(|x, y| x.total_cmp(y))
            .unwrap_or_default()
    }
}

impl History {
    pub fn new(value: VarValue) -> Self {
        Self(vec![Change {
            t: 0.0,
            duration: 0.0,
            tween: default(),
            value,
        }])
    }
    pub fn push(&mut self, change: Change) {
        self.0.push(change)
    }
    pub fn find_value(&self, t: f32) -> Result<VarValue> {
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
    pub fn duration(&self) -> f32 {
        self.0
            .last()
            .map(|x| x.total_duration())
            .unwrap_or_default()
    }
    pub fn get_last(&self) -> Option<VarValue> {
        self.0.last().and_then(|x| Some(x.value.clone()))
    }
}

impl Change {
    pub fn new(value: VarValue) -> Self {
        Self {
            t: default(),
            duration: default(),
            tween: default(),
            value,
        }
    }
    pub fn set_tween(mut self, tween: Tween) -> Self {
        self.tween = tween;
        self
    }
    pub fn set_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }
    pub fn set_t(mut self, t: f32) -> Self {
        self.t = t;
        self
    }
    pub fn total_duration(&self) -> f32 {
        self.t + self.duration
    }
}

impl Tween {
    pub fn f(&self, a: &VarValue, b: &VarValue, t: f32, over: f32) -> Result<VarValue> {
        let t = t / over;
        if t > 1.0 {
            return Ok(b.clone());
        }
        if t < 0.0 {
            return Ok(a.clone());
        }
        let t = match self {
            Tween::Linear => tween::Tweener::linear(0.0, 1.0, 1.0).move_to(t),
            Tween::QuartOut => tween::Tweener::quart_out(0.0, 1.0, 1.0).move_to(t),
            Tween::QuartIn => tween::Tweener::quart_in(0.0, 1.0, 1.0).move_to(t),
            Tween::QuartInOut => tween::Tweener::quart_in_out(0.0, 1.0, 1.0).move_to(t),
            Tween::QuadOut => tween::Tweener::quad_out(0.0, 1.0, 1.0).move_to(t),
            Tween::QuadIn => tween::Tweener::quad_in(0.0, 1.0, 1.0).move_to(t),
            Tween::QuadInOut => tween::Tweener::quad_in_out(0.0, 1.0, 1.0).move_to(t),
            Tween::CubicIn => tween::Tweener::cubic_in(0.0, 1.0, 1.0).move_to(t),
            Tween::CubicOut => tween::Tweener::cubic_out(0.0, 1.0, 1.0).move_to(t),
            Tween::BackIn => tween::Tweener::back_in(0.0, 1.0, 1.0).move_to(t),
        };
        let v = match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => VarValue::Float(*a + (*b - *a) * t),
            (VarValue::Int(a), VarValue::Int(b)) => {
                VarValue::Int(((*a + (*b - *a)) as f32 * t) as i32)
            }
            (VarValue::Vec2(a), VarValue::Vec2(b)) => VarValue::Vec2(*a + (*b - *a) * t),
            (VarValue::Color(a), VarValue::Color(b)) => {
                let mut sub = *b;
                sub.set_r(b.r() - a.r());
                sub.set_g(b.g() - a.g());
                sub.set_b(b.b() - a.b());
                VarValue::Color(*a + sub * t)
            }
            (VarValue::String(a), VarValue::String(b)) => VarValue::String(match t > 0.5 {
                true => a.into(),
                false => b.into(),
            }),
            (VarValue::Bool(a), VarValue::Bool(b)) => VarValue::Bool(match t > 0.5 {
                true => *a,
                false => *b,
            }),
            _ => panic!("Tweening not supported for {a:?} and {b:?}"),
        };
        Ok(v)
    }
}
