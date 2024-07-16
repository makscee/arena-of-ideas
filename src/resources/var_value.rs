use super::*;

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq, AsRefStr)]
pub enum VarValue {
    #[default]
    None,
    Int(i32),
    Float(f32),
    Vec2(Vec2),
    String(String),
    Cstr(Cstr),
    Bool(bool),
    Faction(Faction),
    Color(Color),
    Entity(Entity),
    GID(GID),
    List(Vec<VarValue>),
}

impl VarValue {
    pub fn get_int(&self) -> Result<i32> {
        match self {
            VarValue::None => Ok(0),
            VarValue::Int(v) => Ok(*v),
            VarValue::Float(v) => Ok(*v as i32),
            VarValue::Bool(v) => Ok(*v as i32),
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
            VarValue::Bool(v) => Ok(*v as i32 as f32),
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
            VarValue::Bool(v) => Ok(*v),
            VarValue::Int(v) => Ok(*v > 0),
            VarValue::Float(v) => Ok(*v > 0.0),
            VarValue::String(v) => Ok(!v.is_empty()),
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
            VarValue::Vec2(v) => Ok(*v),
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
    pub fn get_color(&self) -> Result<Color> {
        match self {
            VarValue::None => Ok(Color::FUCHSIA),
            VarValue::Color(v) => Ok(*v),
            _ => Err(anyhow!("Color not supported by {self:?}")),
        }
    }
    pub fn get_color_list(&self) -> Result<Vec<Color>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_color().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_color()?])
    }
    pub fn get_entity(&self) -> Result<Entity> {
        match self {
            VarValue::Entity(v) => Ok(*v),
            _ => Err(anyhow!("Entity not supported by {self:?}")),
        }
    }
    pub fn get_entity_list(&self) -> Result<Vec<Entity>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_entity().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_entity()?])
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
    pub fn get_cstr(&self) -> Result<Cstr> {
        match self {
            VarValue::None => Ok(default()),
            VarValue::Cstr(v) => Ok(v.clone()),
            _ => Err(anyhow!("String not supported by {self:?}")),
        }
    }
    pub fn get_cstr_list(&self) -> Result<Vec<Cstr>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_cstr().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_cstr()?])
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
    pub fn get_gid(&self) -> Result<GID> {
        match self {
            VarValue::GID(v) => Ok(*v),
            _ => Err(anyhow!("Faction not supported by {self:?}")),
        }
    }
    pub fn get_gid_list(&self) -> Result<Vec<GID>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_gid().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_gid()?])
    }

    pub fn sum(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a + b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a + b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a + *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(b + *a as f32)),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a || *b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + *b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(VarValue::String(a.to_owned() + b)),
            _ => Err(anyhow!("{a:?} + {b:?} not supported")),
        }
    }
    pub fn sub(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a - b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a - b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a - *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(*a as f32 - b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a - *b)),
            _ => Err(anyhow!("{a:?} - {b:?} not supported")),
        }
    }
    pub fn mul(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a * b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a * b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a * *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(b * *a as f32)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::Vec2(a), VarValue::Float(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::Float(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            _ => Err(anyhow!("{a:?} * {b:?} not supported")),
        }
    }
    pub fn div(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        if VarValue::Int(0).eq(b) {
            return Err(anyhow!("{a:?} / {b:?} division by zero"));
        }
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a / b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a / b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a / *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(*a as f32 / b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a / *b)),
            (VarValue::Vec2(a), VarValue::Float(b)) => Ok(VarValue::Vec2(*a / *b)),
            _ => Err(anyhow!("{a:?} / {b:?} not supported")),
        }
    }
    pub fn compare(a: &VarValue, b: &VarValue) -> Result<Ordering> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(a.total_cmp(b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(a.cmp(b)),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(a.cmp(b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(a.cmp(b)),
            _ => Err(anyhow!("Comparing {a:?} and {b:?} not supported")),
        }
    }
    pub fn min(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a.min(*b))),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(*(a.min(b)))),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a && *b)),
            _ => Err(anyhow!("Comparing {a:?} and {b:?} not supported")),
        }
    }
    pub fn max(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a.max(*b))),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(*(a.max(b)))),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a || *b)),
            _ => Err(anyhow!("Comparing {a:?} and {b:?} not supported")),
        }
    }
    pub fn abs(self) -> Result<VarValue> {
        match self {
            VarValue::Float(x) => Ok(VarValue::Float(x.abs())),
            VarValue::Int(x) => Ok(VarValue::Int(x.abs())),
            VarValue::Vec2(x) => Ok(VarValue::Vec2(x.abs())),
            _ => Err(anyhow!("Abs {self:?} not supported")),
        }
    }
}

impl ToCstr for VarValue {
    fn cstr(&self) -> Cstr {
        match self {
            VarValue::None => self.as_ref().cstr(),
            VarValue::Int(v) => format!("{v}").cstr(),
            VarValue::Float(v) => format!("{v}").cstr(),
            VarValue::Vec2(v) => format!("{v}").cstr(),
            VarValue::String(v) => format!("{v}").cstr(),
            VarValue::Cstr(v) => v.clone(),
            VarValue::Bool(v) => format!("{v}").cstr(),
            VarValue::Faction(v) => format!("{v}").cstr(),
            VarValue::Color(v) => format!("{v:?}").cstr(),
            VarValue::Entity(v) => format!("{v:?}").cstr(),
            VarValue::GID(v) => v.to_string().cstr(),
            VarValue::List(list) => {
                Cstr::join_vec(list.into_iter().map(|v| v.cstr()).collect_vec())
                    .join(&" + ".cstr())
                    .take()
            }
        }
    }
}

impl From<i32> for VarValue {
    fn from(value: i32) -> Self {
        VarValue::Int(value)
    }
}
impl From<usize> for VarValue {
    fn from(value: usize) -> Self {
        VarValue::Int(value as i32)
    }
}
impl From<f32> for VarValue {
    fn from(value: f32) -> Self {
        VarValue::Float(value)
    }
}
impl From<bool> for VarValue {
    fn from(value: bool) -> Self {
        VarValue::Bool(value)
    }
}
impl From<String> for VarValue {
    fn from(value: String) -> Self {
        VarValue::String(value)
    }
}
impl From<Vec2> for VarValue {
    fn from(value: Vec2) -> Self {
        VarValue::Vec2(value)
    }
}
impl From<Pos2> for VarValue {
    fn from(value: Pos2) -> Self {
        VarValue::Vec2(vec2(value.x, value.y))
    }
}
impl From<Entity> for VarValue {
    fn from(value: Entity) -> Self {
        VarValue::Entity(value)
    }
}
impl From<Color> for VarValue {
    fn from(value: Color) -> Self {
        VarValue::Color(value)
    }
}
impl From<Color32> for VarValue {
    fn from(value: Color32) -> Self {
        VarValue::Color(value.to_color())
    }
}
impl From<Faction> for VarValue {
    fn from(value: Faction) -> Self {
        VarValue::Faction(value)
    }
}
impl From<Cstr> for VarValue {
    fn from(value: Cstr) -> Self {
        VarValue::Cstr(value)
    }
}
impl<T> From<Vec<T>> for VarValue
where
    T: Into<VarValue>,
{
    fn from(value: Vec<T>) -> Self {
        VarValue::List(value.into_iter().map(|v| v.into()).collect_vec())
    }
}
