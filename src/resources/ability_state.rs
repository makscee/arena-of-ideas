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
        self.vars
            .get_mut(house)
            .unwrap()
            .get_mut(ability)
            .unwrap()
            .insert(var, value);
    }
}
