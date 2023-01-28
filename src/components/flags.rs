use super::*;

#[derive(Default)]
pub struct FlagsComponent(HashSet<Flag>);

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Flag {
    DamageImmune,
}

impl FlagsComponent {
    pub fn has_flag(&self, flag: &Flag) -> bool {
        self.0.contains(flag)
    }

    pub fn add_flag(&mut self, flag: Flag) {
        self.0.insert(flag);
    }

    pub fn remove_flag(&mut self, flag: &Flag) {
        self.0.remove(flag);
    }
}
