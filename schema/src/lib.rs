mod action;
mod error;
mod event;
mod expression;
mod fusion;
mod inject;
mod macro_fn;
#[allow(dead_code)]
mod nodes;
mod painter_action;
mod trigger;
mod var_name;
mod var_value;

pub use action::*;
pub use error::*;
pub use event::*;
pub use expression::*;
pub use fusion::*;
pub use inject::*;
pub use macro_fn::*;
pub use painter_action::*;
pub use trigger::*;
pub use var_name::*;
pub use var_value::*;

pub use glam::Vec2;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
pub use utils::*;

pub trait StringData: Sized {
    fn inject_data(&mut self, data: &str);
    fn get_data(&self) -> String;
}
impl<T: Serialize + DeserializeOwned> StringData for T {
    fn inject_data(&mut self, data: &str) {
        match ron::from_str(data) {
            Ok(v) => *self = v,
            Err(e) => log::error!("Deserialize error: {e}"),
        }
    }
    fn get_data(&self) -> String {
        ron::to_string(self).unwrap()
    }
}

pub type NodeChildren<T> = Vec<T>;
pub type NodeComponent<T> = Option<T>;
