mod representation;
mod status;
mod var_state;
mod var_state_delta;

use super::*;
use ecolor::Color32;
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

impl Into<Color32> for HexColor {
    fn into(self) -> Color32 {
        let c: Color = self.into();
        let c = c.as_rgba_u8();
        Color32::from_rgb(c[0], c[1], c[2])
    }
}

impl Default for HexColor {
    fn default() -> Self {
        Self("#ff00ff".into())
    }
}
