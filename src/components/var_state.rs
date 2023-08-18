use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct VarState(HashMap<VarName, History>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct History(Vec<Change>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Change {
    pub t: f32,
    #[serde(default)]
    pub duration: f32, // over what period the change will be applied
    #[serde(default)]
    pub tween: Tween,
    pub value: VarValue,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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
    pub fn get_value(&self, var: VarName, t: f32) -> Result<VarValue> {
        self.0.get(&var).context("No key in state")?.find_value(t)
    }
}

impl History {
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
            _ => panic!("Tweening not supported for {a:?} and {b:?}"),
        };
        Ok(v)
    }
}
