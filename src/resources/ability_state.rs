use super::*;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AbilitiesState {
    pub vars: HashMap<HouseName, HashMap<String, Vars>>,
}

impl AbilitiesState {
    pub fn get_var(&self, house: &HouseName, ability: &str, var: &VarName) -> Option<&Var> {
        self.get_vars(house, ability).and_then(|x| x.try_get(var))
    }

    pub fn get_vars(&self, house: &HouseName, ability: &str) -> Option<&Vars> {
        self.vars.get(house).and_then(|x| x.get(ability))
    }
}
