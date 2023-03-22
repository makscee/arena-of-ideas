use super::*;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AbilitiesState {
    pub vars: HashMap<HouseName, HashMap<String, Vars>>,
}

impl AbilitiesState {
    pub fn get_vars(&self, house: &HouseName, ability: &str) -> Option<&Vars> {
        self.vars.get(house).and_then(|x| x.get(ability))
    }

    pub fn set_var(&mut self, house: &HouseName, ability: &str, var: VarName, value: Var) {
        let mut house_map = self.vars.remove(house).unwrap_or_default();
        let mut vars = house_map.remove(ability).unwrap_or_default();
        vars.insert(var, value);
        house_map.insert(ability.to_string(), vars);
        self.vars.insert(*house, house_map);
    }
}
