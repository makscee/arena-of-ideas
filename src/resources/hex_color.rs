use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HexColor(pub String);

impl From<HexColor> for Color {
    fn from(value: HexColor) -> Self {
        Color::hex(value.0).unwrap()
    }
}

impl From<HexColor> for Color32 {
    fn from(value: HexColor) -> Self {
        let c: Color = value.into();
        let c = c.as_rgba_u8();
        Color32::from_rgb(c[0], c[1], c[2])
    }
}

impl Default for HexColor {
    fn default() -> Self {
        Self("#ff00ff".into())
    }
}
