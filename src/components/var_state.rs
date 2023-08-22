use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, Reflect)]
pub struct VarState(HashMap<VarName, History>);

#[derive(Serialize, Deserialize, Clone, Debug, Default, Reflect)]
pub struct History(Vec<Change>);

#[derive(Serialize, Deserialize, Clone, Debug, Reflect)]
pub struct Change {
    pub t: f32,
    #[serde(default)]
    pub duration: f32, // over what period the change will be applied
    #[serde(default)]
    pub tween: Tween,
    pub value: VarValue,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Reflect)]
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
    pub fn push_back(&mut self, var: VarName, mut change: Change) {
        change.t += self.duration();
        self.0.entry(var).or_insert(default()).push(change);
    }
    pub fn insert(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.0.insert(var, History::new(value));
        self
    }
    pub fn get_value(&self, var: VarName, t: f32) -> Result<VarValue> {
        self.0.get(&var).context("No key in state")?.find_value(t)
    }
    pub fn get_value_from_world(entity: Entity, var: VarName, world: &World) -> Result<VarValue> {
        let t = world
            .get_resource::<Time>()
            .context("Time not found")?
            .elapsed_seconds();
        world.get::<VarState>(entity).unwrap().get_value(var, t)
    }
    pub fn find_value(mut entity: Entity, var: VarName, t: f32, world: &World) -> Result<VarValue> {
        let mut result = None;
        loop {
            if let Some(state) = world.get::<VarState>(entity) {
                if let Ok(value) = state.get_value(var, t) {
                    result = Some(value);
                    break;
                }
            }
            if result.is_none() {
                if let Some(parent) = world.get::<Parent>(entity) {
                    entity = parent.get();
                    continue;
                }
            }
            break;
        }
        result.context("Var was not found")
    }
    pub fn duration(&self) -> f32 {
        self.0
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
        let i = self.0.partition_point(|x| x.t < t);
        if i == 0 {
            return Err(anyhow!("First change not reached"));
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
        self.0.last().map(|x| x.t + x.duration).unwrap_or_default()
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
            (VarValue::Vec2(a), VarValue::Vec2(b)) => VarValue::Vec2(*a + (*b - *a) * t),
            _ => panic!("Tweening not supported for {a:?} and {b:?}"),
        };
        Ok(v)
    }
}
