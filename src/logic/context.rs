use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogicEffectContext {
    pub owner: Id,
    pub creator: Id,
    pub target: Id,
    pub vars: HashMap<VarName, i32>,
    pub color: Rgba<f32>,
}

impl LogicEffectContext {
    pub fn get_id(&self, who: Who) -> Id {
        match who {
            Who::Owner => self.owner,
            Who::Creator => self.creator,
            Who::Target => self.target,
        }
    }
}
