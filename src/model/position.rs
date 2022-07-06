use super::*;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub side: Faction,
    pub x: Coord,
    pub height: Coord,
}

impl Position {
    pub fn zero(side: Faction) -> Self {
        Self {
            side,
            x: 0,
            height: 0,
        }
    }

    pub fn to_world(&self) -> Vec2<R32> {
        let offset = match self.side {
            Faction::Player => -1.0,
            Faction::Enemy => 1.0,
        };
        vec2(self.x, self.height).map(|x| r32(x as f32 + offset))
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
