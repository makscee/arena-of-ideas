use super::*;

#[derive(FromRepr, AsRefStr, EnumIter, PartialEq, Clone, Copy, Display, Default, Debug)]
#[repr(u8)]
pub enum Rarity {
    #[default]
    Common,
    Rare,
    Epic,
    Legendary,
}

const RARITY_COLORS: [Color32; 4] = [
    hex_color_noa!("#B0BEC5"),
    hex_color_noa!("#0277BD"),
    hex_color_noa!("#AB47BC"),
    hex_color_noa!("#F57C00"),
];

impl Rarity {
    pub fn color(self) -> Color32 {
        RARITY_COLORS[self as usize]
    }
}

impl ToCstr for Rarity {
    fn cstr(&self) -> Cstr {
        self.to_string().cstr_c(self.color())
    }
}

impl From<u8> for Rarity {
    fn from(v: u8) -> Self {
        Self::from_repr(v).unwrap_or_default()
    }
}
impl Into<u8> for Rarity {
    fn into(self) -> u8 {
        self as u8
    }
}
