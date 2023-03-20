use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Default, Clone)]
pub struct HouseComponent {
    pub houses: HashSet<HouseName>,
}

impl HouseComponent {
    pub fn new(houses: HashSet<HouseName>) -> Self {
        Self { houses }
    }
}

impl VarsProvider for HouseComponent {
    fn extend_vars(&self, vars: &mut Vars, resources: &Resources) {
        let colors = self
            .houses
            .iter()
            .map(|house| resources.houses.get(house).unwrap().color)
            .collect_vec();
        vars.insert(VarName::HouseColor1, Var::Color(colors[0]));
        if colors.len() > 1 {
            vars.insert(VarName::HouseColor2, Var::Color(colors[1]));
        }
        if colors.len() > 2 {
            vars.insert(VarName::HouseColor3, Var::Color(colors[2]));
        }
    }
}
