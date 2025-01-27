use super::*;

struct House {
    pub name: String,
    pub color: Option<HouseColor>,
    pub abilities: Vec<Ability>,
}

struct HouseColor {
    pub color: String,
}

struct Ability {
    pub name: String,
    pub description: Option<AbilityDescription>,
    pub units: Vec<Unit>,
}

struct AbilityDescription {
    pub description: String,
    pub effect: Option<AbilityEffect>,
}

struct AbilityEffect {
    pub actions: Actions,
}

struct Team {
    pub name: String,
    pub houses: Vec<House>,
    pub units: Vec<Unit>,
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

struct Unit {
    pub name: String,
    pub slot: Option<UnitSlot>,
    pub stats: Option<UnitStats>,
    pub description: Option<UnitDescription>,
    pub representation: Option<Representation>,
    pub house_link: Vec<UnitHouseLink>,
}

struct UnitStats {
    pub pwr: i32,
    pub hp: i32,
}

struct UnitSlot {
    pub slot: i32,
}

struct UnitHouseLink {
    pub name: String,
}

struct UnitDescription {
    pub description: String,
    pub reaction: Option<Reaction>,
}

struct Status {
    pub name: String,
    pub description: Option<StatusDescription>,
    pub representation: Option<Representation>,
}

struct StatusDescription {
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
    pub children: Vec<Box<Representation>>,
}
