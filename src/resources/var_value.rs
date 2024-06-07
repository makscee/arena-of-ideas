use super::*;

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum VarValue {
    #[default]
    None,
    Int(i32),
    Float(f32),
    Vec2(Vec2),
    String(String),
    Bool(bool),
    Faction(Faction),
    List(Vec<VarValue>),
}

impl VarValue {
    pub fn get_int(&self) -> Result<i32> {
        match self {
            VarValue::None => Ok(0),
            VarValue::Int(v) => Ok(*v),
            VarValue::Float(v) => Ok(*v as i32),
            _ => Err(anyhow!("Int not supported by {self:?}")),
        }
    }
    pub fn get_int_list(&self) -> Result<Vec<i32>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_int().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_int()?])
    }
    pub fn get_float(&self) -> Result<f32> {
        match self {
            VarValue::None => Ok(0.0),
            VarValue::Int(v) => Ok(*v as f32),
            VarValue::Float(v) => Ok(*v),
            _ => Err(anyhow!("Float not supported by {self:?}")),
        }
    }
    pub fn get_float_list(&self) -> Result<Vec<f32>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_float().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_float()?])
    }
    pub fn get_bool(&self) -> Result<bool> {
        match self {
            VarValue::None => Ok(false),
            VarValue::Bool(value) => Ok(*value),
            VarValue::Int(value) => Ok(*value > 0),
            VarValue::Float(value) => Ok(*value > 0.0),
            VarValue::String(value) => Ok(!value.is_empty()),
            _ => Err(anyhow!("Bool not supported by {self:?}")),
        }
    }
    pub fn get_bool_list(&self) -> Result<Vec<bool>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_bool().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_bool()?])
    }
    pub fn get_vec2(&self) -> Result<Vec2> {
        match self {
            VarValue::None => Ok(Vec2::ZERO),
            VarValue::Int(v) => Ok(vec2(*v as f32, *v as f32)),
            VarValue::Float(v) => Ok(vec2(*v, *v)),
            _ => Err(anyhow!("Vec2 not supported by {self:?}")),
        }
    }
    pub fn get_vec2_list(&self) -> Result<Vec<Vec2>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_vec2().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_vec2()?])
    }
    pub fn get_string(&self) -> Result<String> {
        match self {
            VarValue::None => Ok(default()),
            VarValue::Int(v) => Ok(v.to_string()),
            VarValue::Float(v) => Ok(v.to_string()),
            VarValue::Vec2(v) => Ok(v.to_string()),
            VarValue::String(v) => Ok(v.clone()),
            _ => Err(anyhow!("String not supported by {self:?}")),
        }
    }
    pub fn get_string_list(&self) -> Result<Vec<String>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_string().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_string()?])
    }
    pub fn get_faction(&self) -> Result<Faction> {
        match self {
            VarValue::Faction(v) => Ok(*v),
            _ => Err(anyhow!("Faction not supported by {self:?}")),
        }
    }
    pub fn get_faction_list(&self) -> Result<Vec<Faction>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_faction().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_faction()?])
    }

    pub fn sum(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a + b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a + b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a + *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(b + *a as f32)),
            // (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a || *b)),
            // (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + *b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(VarValue::String(a.to_owned() + b)),
            _ => Err(anyhow!("{a:?} + {b:?} not supported")),
        }
    }
}
