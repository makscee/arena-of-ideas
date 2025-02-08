use std::hash::{DefaultHasher, Hash, Hasher};

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Descriptions {
    expressions: Vec<(Expression, String)>,
}

static DESCRIPTIONS: OnceCell<HashMap<u64, String>> = OnceCell::new();

impl Descriptions {
    pub fn set(self) {
        dbg!(&self);
        let mut map: HashMap<u64, String> = default();
        for (x, text) in self.expressions {
            let mut state = DefaultHasher::new();
            x.hash(&mut state);
            map.insert(state.finish(), text);
        }
        DESCRIPTIONS.set(map).unwrap();
    }
    pub fn get(key: impl Hash) -> Option<&'static String> {
        let mut state = DefaultHasher::new();
        key.hash(&mut state);
        DESCRIPTIONS.get().unwrap().get(&state.finish())
    }
}
