use super::*;

#[derive(FromRepr, AsRefStr, EnumIter, PartialEq, Clone, Copy)]
#[repr(i32)]
pub enum Rarity {
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
}
