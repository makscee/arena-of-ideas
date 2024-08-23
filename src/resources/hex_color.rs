use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HexColor(pub String);

impl From<HexColor> for Color32 {
    fn from(value: HexColor) -> Self {
        ecolor::HexColor::from_str(&value.0).unwrap().color()
    }
}

impl From<HexColor> for Color {
    fn from(value: HexColor) -> Self {
        let c: Color32 = value.into();
        c.to_color()
    }
}

impl Default for HexColor {
    fn default() -> Self {
        Self("#ff00ff".into())
    }
}
