use super::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct VarStateDelta {
    pub state: VarState,
}

impl VarStateDelta {
    pub fn process(&self, var: VarName, value: VarValue, t: f32) -> VarValue {
        if let Ok(delta) = self.state.get_value_at(var, t) {
            VarValue::sum(&value, &delta).unwrap()
        } else {
            value
        }
    }

    pub fn process_last(&self, var: VarName, value: VarValue) -> VarValue {
        if let Ok(delta) = self.state.get_value_last(var) {
            VarValue::sum(&value, &delta).unwrap()
        } else {
            value
        }
    }

    pub fn need_update(&self, var: VarName, new: &VarValue) -> bool {
        match self.state.get_value_last(var) {
            Ok(value) => !value.eq(new),
            Err(_) => true,
        }
    }
}
