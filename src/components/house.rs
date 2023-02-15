use super::*;

pub struct HouseComponent {
    pub houses: HashSet<HouseName>,
}

impl HouseComponent {
    pub fn new(houses: Vec<HouseName>) -> Self {
        Self {
            houses: HashSet::from_iter(houses),
        }
    }
}
