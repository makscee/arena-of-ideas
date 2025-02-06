use super::*;

pub struct UnitCard {
    name: String,
    description: String,
    house: String,
    house_color: Color32,
    vars: HashMap<VarName, VarValue>,
}
