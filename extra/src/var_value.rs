use super::*;

#[derive(Clone, Default, Serialize, Deserialize, Debug, AsRefStr)]
pub enum VarValue {
    #[default]
    None,
    i32(i32),
    f32(f32),
    Vec2(Vec2),
    String(String),
    Bool(bool),
    Color(Color),
    Entity(Entity),
    u64(u64),
    List(Vec<VarValue>),
}

impl VarValue {
    pub fn get_list(&self) -> Result<Vec<VarValue>> {
        match self {
            VarValue::List(list) => Ok(list.clone()),
            _ => Err(anyhow!("List not supported by {self}")),
        }
    }
    pub fn get_int(&self) -> Result<i32> {
        match self {
            VarValue::None => Ok(0),
            VarValue::i32(v) => Ok(*v),
            VarValue::f32(v) => Ok(*v as i32),
            VarValue::Bool(v) => Ok(*v as i32),
            _ => Err(anyhow!("i32 not supported by {self}")),
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
            VarValue::i32(v) => Ok(*v as f32),
            VarValue::f32(v) => Ok(*v),
            VarValue::Bool(v) => Ok(*v as i32 as f32),
            _ => Err(anyhow!("f32 not supported by {self}")),
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
            VarValue::i32(v) => Ok(*v > 0),
            VarValue::f32(v) => Ok(*v > 0.0),
            VarValue::String(v) => Ok(!v.is_empty()),
            _ => Err(anyhow!("Bool not supported by {self}")),
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
            VarValue::i32(v) => Ok(vec2(*v as f32, *v as f32)),
            VarValue::f32(v) => Ok(vec2(*v, *v)),
            VarValue::Vec2(v) => Ok(*v),
            _ => Err(anyhow!("Vec2 not supported by {self}")),
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
            VarValue::None => Ok(BEVY_MISSING_COLOR.into()),
            VarValue::Color(v) => Ok(*v),
            _ => Err(anyhow!("Color not supported by {self}")),
        }
    }
    pub fn get_color32(&self) -> Result<Color32> {
        match self {
            VarValue::None => Ok(Color::from(BEVY_MISSING_COLOR).c32()),
            VarValue::Color(v) => Ok(v.c32()),
            _ => Err(anyhow!("Color not supported by {self}")),
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
    pub fn get_color32_list(&self) -> Result<Vec<Color32>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_color32().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_color32()?])
    }
    pub fn get_entity(&self) -> Result<Entity> {
        match self {
            VarValue::Entity(v) => Ok(*v),
            _ => Err(anyhow!("Entity not supported by {self}")),
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
            VarValue::i32(v) => Ok(v.to_string()),
            VarValue::f32(v) => Ok(v.to_string()),
            VarValue::Vec2(v) => Ok(v.to_string()),
            VarValue::String(v) => Ok(v.clone()),
            _ => Err(anyhow!("String not supported by {self}")),
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
    pub fn get_u64(&self) -> Result<u64> {
        match self {
            VarValue::u64(v) => Ok(*v),
            _ => Err(anyhow!("u64 not supported by {self}")),
        }
    }
    pub fn get_gid_list(&self) -> Result<Vec<u64>> {
        match self {
            VarValue::List(list) => {
                return Ok(list
                    .into_iter()
                    .filter_map(|v| v.get_u64().ok())
                    .collect_vec());
            }
            _ => {}
        }
        Ok(vec![self.get_u64()?])
    }

    pub fn sum(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::String(a), ..) => {
                Ok(VarValue::String(a.to_owned() + b.get_string()?.as_str()))
            }
            (.., VarValue::String(b)) => Ok(VarValue::String(a.get_string()? + b.as_str())),
            (VarValue::f32(a), ..) => Ok(VarValue::f32(a + b.get_float()?)),
            (.., VarValue::f32(b)) => Ok(VarValue::f32(a.get_float()? + *b)),
            (VarValue::i32(a), ..) => Ok(VarValue::i32(a + b.get_int()?)),
            (.., VarValue::i32(b)) => Ok(VarValue::i32(a.get_int()? + *b)),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a || *b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + *b)),
            _ => Err(anyhow!("{a} + {b} not supported")),
        }
    }
    pub fn sub(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a - b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(a - b)),
            (VarValue::f32(a), VarValue::i32(b)) => Ok(VarValue::f32(a - *b as f32)),
            (VarValue::i32(a), VarValue::f32(b)) => Ok(VarValue::f32(*a as f32 - b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a - *b)),
            _ => Err(anyhow!("{a} - {b} not supported")),
        }
    }
    pub fn mul(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a * b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(a * b)),
            (VarValue::f32(a), VarValue::i32(b)) => Ok(VarValue::f32(a * *b as f32)),
            (VarValue::i32(a), VarValue::f32(b)) => Ok(VarValue::f32(b * *a as f32)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::Vec2(a), VarValue::f32(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::f32(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            _ => Err(anyhow!("{a} * {b} not supported")),
        }
    }
    pub fn div(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        if VarValue::i32(0).eq(b) {
            return Err(anyhow!("{a} / {b} division by zero"));
        }
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a / b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(a / b)),
            (VarValue::f32(a), VarValue::i32(b)) => Ok(VarValue::f32(a / *b as f32)),
            (VarValue::i32(a), VarValue::f32(b)) => Ok(VarValue::f32(*a as f32 / b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a / *b)),
            (VarValue::Vec2(a), VarValue::f32(b)) => Ok(VarValue::Vec2(*a / *b)),
            _ => Err(anyhow!("{a} / {b} not supported")),
        }
    }
    pub fn compare(a: &VarValue, b: &VarValue) -> Result<Ordering> {
        match (a, b) {
            (VarValue::None, VarValue::None) => Ok(Ordering::Equal),
            (VarValue::f32(a), VarValue::f32(b)) => Ok(a.total_cmp(b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(a.cmp(b)),
            (VarValue::u64(a), VarValue::u64(b)) => Ok(a.cmp(b)),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(a.cmp(b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(a.get_text().cmp(&b.get_text())),
            _ => Err(anyhow!("Comparing {a} and {b} not supported")),
        }
    }
    pub fn min(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a.min(*b))),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(*(a.min(b)))),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a && *b)),
            _ => Err(anyhow!("Comparing {a} and {b} not supported")),
        }
    }
    pub fn max(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a.max(*b))),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(*(a.max(b)))),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a || *b)),
            _ => Err(anyhow!("Comparing {a} and {b} not supported")),
        }
    }
    pub fn abs(self) -> Result<VarValue> {
        match self {
            VarValue::f32(x) => Ok(VarValue::f32(x.abs())),
            VarValue::i32(x) => Ok(VarValue::i32(x.abs())),
            VarValue::Vec2(x) => Ok(VarValue::Vec2(x.abs())),
            _ => Err(anyhow!("Abs {self} not supported")),
        }
    }
}

impl std::hash::Hash for VarValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            VarValue::None => core::mem::discriminant(self).hash(state),
            VarValue::f32(v) => (*v).ord().hash(state),
            VarValue::i32(v) => (*v).hash(state),
            VarValue::Vec2(Vec2 { x, y }) => {
                (*x).ord().hash(state);
                (*y).ord().hash(state);
            }
            VarValue::Bool(v) => (*v).hash(state),
            VarValue::String(v) => (*v).hash(state),
            VarValue::Entity(v) => (*v).to_bits().hash(state),
            VarValue::List(v) => {
                for v in v {
                    (*v).hash(state)
                }
            }
            VarValue::Color(v) => {
                let c = v.c32();
                c.r().hash(state);
                c.g().hash(state);
                c.b().hash(state);
            }
            VarValue::u64(v) => (*v).hash(state),
        };
    }
}

impl ToCstr for VarValue {
    fn cstr(&self) -> Cstr {
        match self {
            VarValue::Entity(e) => entity_name(*e),
            VarValue::Color(c) => {
                let c = c.c32();
                c.to_hex().cstr_c(c)
            }
            _ => self.to_string().cstr(),
        }
    }
}
impl std::fmt::Display for VarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarValue::None => write!(f, "None"),
            VarValue::i32(v) => write!(f, "{}", v),
            VarValue::f32(v) => write!(f, "{:.2}", v),
            VarValue::Vec2(v) => write!(f, "{:.2}, {:.2}", v.x, v.y),
            VarValue::String(v) => write!(f, "{}", v),
            VarValue::Bool(v) => write!(f, "{}", v),
            VarValue::Color(v) => write!(f, "{}", v.c32().to_hex()),
            VarValue::Entity(v) => write!(f, "{}", v),
            VarValue::u64(v) => write!(f, "{}", v),
            VarValue::List(v) => write!(f, "{}", v.iter().join(", ")),
        }
    }
}

impl From<i32> for VarValue {
    fn from(value: i32) -> Self {
        VarValue::i32(value)
    }
}
impl From<usize> for VarValue {
    fn from(value: usize) -> Self {
        VarValue::i32(value as i32)
    }
}
impl From<f32> for VarValue {
    fn from(value: f32) -> Self {
        VarValue::f32(value)
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
impl From<&str> for VarValue {
    fn from(value: &str) -> Self {
        VarValue::String(value.to_string())
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
impl From<u64> for VarValue {
    fn from(value: u64) -> Self {
        VarValue::u64(value)
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

impl PartialEq for VarValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::i32(l0), Self::i32(r0)) => l0 == r0,
            (Self::f32(l0), Self::f32(r0)) => l0 == r0,
            (Self::Vec2(l0), Self::Vec2(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0.get_text() == r0.get_text(),
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Color(l0), Self::Color(r0)) => l0 == r0,
            (Self::Entity(l0), Self::Entity(r0)) => l0 == r0,
            (Self::u64(l0), Self::u64(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
