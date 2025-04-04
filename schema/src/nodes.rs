use super::*;

struct Core {
    pub houses: NodeChildren<House>,
}

struct Players {
    pub players: NodeChildren<Player>,
}

struct Incubator {
    pub houses: NodeChildren<House>,
    pub house_colors: NodeChildren<HouseColor>,
    pub units: NodeChildren<Unit>,
    pub unit_descriptions: NodeChildren<UnitDescription>,
    pub unit_stats: NodeChildren<UnitStats>,
    pub abilities: NodeChildren<AbilityMagic>,
    pub ability_descriptions: NodeChildren<AbilityDescription>,
    pub ability_effects: NodeChildren<AbilityEffect>,
    pub statuses: NodeChildren<StatusMagic>,
    pub status_descriptions: NodeChildren<StatusDescription>,
    pub representations: NodeChildren<Representation>,
    pub reactions: NodeChildren<Behavior>,
}

struct Player {
    pub player_name: String,
    pub player_data: NodeComponent<PlayerData>,
    pub identity: NodeComponent<PlayerIdentity>,
    pub active_match: NodeComponent<Match>,
}

struct PlayerData {
    pub pass_hash: Option<String>,
    pub online: bool,
    pub last_login: u64,
}

struct PlayerIdentity {
    pub data: Option<String>,
}

struct House {
    pub house_name: String,
    pub color: NodeComponent<HouseColor>,
    pub action_ability: NodeComponent<AbilityMagic>,
    pub status_ability: NodeComponent<StatusMagic>,
    pub units: NodeChildren<Unit>,
}

struct HouseColor {
    pub color: HexColor,
}

struct AbilityMagic {
    pub ability_name: String,
    pub description: NodeComponent<AbilityDescription>,
}

struct AbilityDescription {
    pub description: String,
    pub effect: NodeComponent<AbilityEffect>,
}

struct AbilityEffect {
    pub actions: Actions,
}

struct StatusMagic {
    pub status_name: String,
    pub description: NodeComponent<StatusDescription>,
    pub representation: NodeComponent<Representation>,
}

struct StatusDescription {
    pub description: String,
    pub behavior: NodeComponent<Behavior>,
}

struct Team {
    pub team_name: String,
    pub houses: NodeChildren<House>,
    pub fusions: NodeChildren<Fusion>,
}

struct Match {
    pub g: i32,
    pub shop_case: NodeChildren<ShopCaseUnit>,
    pub team: NodeComponent<Team>,
}

struct ShopCaseUnit {
    pub unit: u64,
    pub price: i32,
    pub sold: bool,
}

struct Fusion {
    pub units: Vec<String>,
    pub behavior: Vec<(UnitTriggerRef, Vec<UnitActionRef>)>,
    pub slot: i32,
}

struct Unit {
    pub unit_name: String,
    pub description: NodeComponent<UnitDescription>,
}

struct UnitDescription {
    pub description: String,
    pub representation: NodeComponent<Representation>,
    pub stats: NodeComponent<UnitStats>,
    pub behavior: NodeComponent<Behavior>,
}

struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
}

struct Behavior {
    pub reactions: Vec<Reaction>,
}

struct Representation {
    pub material: Material,
}
