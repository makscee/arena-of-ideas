use super::*;

struct House {
    pub name: String,
    pub color: Option<HouseColor>,
    pub action_abilities: Vec<ActionAbility>,
    pub status_abilities: Vec<StatusAbility>,
}

struct HouseColor {
    pub color: String,
}

struct ActionAbility {
    pub name: String,
    pub description: Option<ActionAbilityDescription>,
    pub units: Vec<Unit>,
}

struct ActionAbilityDescription {
    pub description: String,
    pub effect: Option<AbilityEffect>,
}

struct AbilityEffect {
    pub actions: Actions,
}

struct StatusAbility {
    pub name: String,
    pub description: Option<StatusAbilityDescription>,
    pub representation: Option<Representation>,
    pub units: Vec<Unit>,
}

struct StatusAbilityDescription {
    pub description: String,
    pub reaction: Option<Reaction>,
}

struct Team {
    pub name: String,
    pub houses: Vec<House>,
    pub fusions: Vec<Fusion>,
}

struct Match {
    pub g: i32,
    pub last_update: u64,
    pub shop_case: Vec<ShopCaseUnit>,
    pub team: Option<Team>,
}

struct ShopCaseUnit {
    pub unit_id: u64,
    pub price: i32,
    pub sold: bool,
}

struct Fusion {
    pub unit: FusedUnit,
    pub slot: Option<UnitSlot>,
}

struct Unit {
    pub name: String,
    pub stats: Option<UnitStats>,
    pub description: Option<UnitDescription>,
    pub representation: Option<Representation>,
}

struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
    pub dmg: i32,
}

struct UnitSlot {
    pub slot: i32,
}

struct UnitDescription {
    pub description: String,
    pub reaction: Option<Reaction>,
}

struct Reaction {
    pub trigger: Trigger,
    pub actions: Actions,
}

struct Hero {
    pub name: String,
    pub representation: Option<Representation>,
    pub mover: Option<Mover>,
}

struct Mover {
    pub target: Vec2,
    pub from: Vec2,
    pub start_ts: f64,
}

impl Mover {
    pub fn pos(&self, speed: f32) -> Vec2 {
        if self.start_ts == 0.0 {
            return Vec2::ZERO;
        }
        self.from.lerp(self.target, self.t(speed))
    }
    pub fn t(&self, speed: f32) -> f32 {
        let elapsed = (now_seconds() - self.start_ts) as f32;
        let t = (self.target - self.from).length() / speed;
        (elapsed / t).clamp(0.0, 1.0)
    }
}

struct Representation {
    pub material: Material,
}
