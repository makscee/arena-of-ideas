use super::*;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub side: Faction,
    pub x: Coord,
}

impl Position {
    pub fn zero(side: Faction) -> Self {
        Self { side, x: 0 }
    }

    pub fn to_world(&self) -> Vec2<R32> {
        let pos = r32(self.x as f32);
        match self.side {
            Faction::Player => vec2(-pos - r32(1.0), r32(0.0)),
            Faction::Enemy => vec2(pos + r32(1.0), r32(0.0)),
        }
    }

    pub fn to_world_f32(&self) -> Vec2<f32> {
        self.to_world().map(|x| x.as_f32())
    }

    pub fn distance(&self, other: &Self) -> Coord {
        if self.side == other.side {
            (self.x - other.x).abs()
        } else {
            self.x + other.x
        }
    }
}
