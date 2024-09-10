use super::*;

#[derive(Asset, Serialize, Deserialize, Clone, Debug, TypePath, PartialEq)]
pub struct House {
    pub name: String,
    pub color: HexColor,
    #[serde(default)]
    pub abilities: Vec<Ability>,
    #[serde(default)]
    pub statuses: Vec<PackedStatus>,
    #[serde(default)]
    pub summons: Vec<PackedUnit>,
    #[serde(default)]
    pub defaults: HashMap<String, HashMap<VarName, VarValue>>,
}

impl From<THouse> for House {
    fn from(value: THouse) -> Self {
        Self {
            name: value.name,
            color: HexColor(value.color),
            abilities: value
                .abilities
                .into_iter()
                .filter_map(|a| TAbility::find_by_name(a).map(|a| a.into()))
                .collect_vec(),
            statuses: value
                .statuses
                .into_iter()
                .filter_map(|s| TStatus::find_by_name(s).map(|s| s.into()))
                .collect_vec(),
            summons: value
                .summons
                .into_iter()
                .filter_map(|u| TBaseUnit::find_by_name(u).map(|u| u.into()))
                .collect_vec(),
            defaults: ron::from_str(&value.defaults).unwrap(),
        }
    }
}

impl From<House> for THouse {
    fn from(value: House) -> Self {
        Self {
            name: value.name,
            color: value.color.0,
            abilities: value.abilities.into_iter().map(|a| a.name).collect_vec(),
            statuses: value.statuses.into_iter().map(|s| s.name).collect_vec(),
            summons: value.summons.into_iter().map(|u| u.name).collect_vec(),
            defaults: ron::to_string(&value.defaults).unwrap(),
        }
    }
}
