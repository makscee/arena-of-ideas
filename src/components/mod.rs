mod representation;
mod status;
mod var_state;
mod var_state_delta;

use super::*;
pub use representation::*;
pub use status::*;
pub use var_state::*;
pub use var_state_delta::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HexColor(String);

impl Into<Color> for HexColor {
    fn into(self) -> Color {
        Color::hex(&self.0).unwrap()
    }
}

impl Default for HexColor {
    fn default() -> Self {
        Self("#ff00ff".into())
    }
}
