use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Default, Clone)]
pub struct HouseComponent {
    pub houses: HashMap<HouseName, Rgba<f32>>,
}

impl HouseComponent {
    pub fn new(houses: Vec<HouseName>, resources: &Resources) -> Self {
        Self {
            houses: HashMap::from_iter(
                houses
                    .iter()
                    .map(|name| (*name, resources.houses.get(name).unwrap().color)),
            ),
        }
    }
}

impl VarsProvider for HouseComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        let colors = self.houses.iter().map(|(_, color)| color).collect_vec();
        vars.insert(VarName::HouseColor1, Var::Color(*colors[0]));
        if colors.len() > 1 {
            vars.insert(VarName::HouseColor2, Var::Color(*colors[1]));
        }
        if colors.len() > 2 {
            vars.insert(VarName::HouseColor3, Var::Color(*colors[2]));
        }
    }
}
