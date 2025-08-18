mod node_assets;
mod nodes;

use super::*;

pub use node_assets::*;
pub use nodes::*;

pub trait EntityConverter {
    fn to_e(&self) -> Entity;
}

impl EntityConverter for u64 {
    fn to_e(&self) -> Entity {
        Entity::from_bits(*self)
    }
}

impl From<&String> for TNode {
    fn from(value: &String) -> Self {
        ron::from_str::<SerdeWrapper<TNode>>(value).unwrap().0
    }
}
impl Into<String> for TNode {
    fn into(self) -> String {
        self.to_ron()
    }
}
