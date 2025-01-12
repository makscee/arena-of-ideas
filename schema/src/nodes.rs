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
    pub data: String,
}

struct AbilityEffect {
    pub data: String,
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
}

struct UnitDescription {
    pub description: String,
    pub trigger: Option<UnitTrigger>,
}

struct UnitTrigger {
    pub trigger: Trigger,
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
