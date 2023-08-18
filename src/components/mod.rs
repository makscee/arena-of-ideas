mod representation;
mod var_state;

use super::*;
pub use representation::*;
pub use var_state::*;

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
