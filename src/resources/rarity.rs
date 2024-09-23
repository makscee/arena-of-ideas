use super::*;

#[derive(FromRepr, AsRefStr, EnumIter, PartialEq, Clone, Copy, Display, Default)]
#[repr(i8)]
pub enum Rarity {
    #[default]
    Common,
    Rare,
    Epic,
    Legendary,
}

const RARITY_COLORS: [Color32; 5] = [
    hex_color_noa!("#607D8B"),
    hex_color_noa!("#B0BEC5"),
    hex_color_noa!("#0277BD"),
    hex_color_noa!("#AB47BC"),
    hex_color_noa!("#F57C00"),
];

pub fn rarity_color(i: i8) -> Color32 {
    let i = (i + 1) as usize;
    RARITY_COLORS[i]
}

impl Rarity {
    pub fn color(self) -> Color32 {
        RARITY_COLORS[self as usize + 1]
    }
    pub fn from_int(v: i8) -> Self {
        Self::from_repr(v).unwrap_or_default()
    }
    pub fn from_base(name: &str) -> Self {
        Self::from_int(TBaseUnit::find_by_name(name.into()).unwrap().rarity)
    }
}

impl ToCstr for Rarity {
    fn cstr(&self) -> Cstr {
        self.to_string().cstr_c(self.color())
    }
}
