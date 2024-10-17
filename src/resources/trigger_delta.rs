use super::*;

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter, Default, AsRefStr)]
pub enum DeltaTrigger {
    #[default]
    IncomingDamage,
    Var(VarName),
}

impl DeltaTrigger {
    pub fn catch(&self, event: &Event) -> bool {
        match self {
            DeltaTrigger::IncomingDamage => matches!(event, Event::IncomingDamage { .. }),
            DeltaTrigger::Var(..) => false,
        }
    }
}

impl ShowEditor for DeltaTrigger {
    fn get_variants() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    fn show_children(&mut self, _context: &Context, _world: &mut World, _ui: &mut Ui) {}

    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        default()
    }
    fn show_content(&mut self, _context: &Context, _world: &mut World, ui: &mut Ui) {
        match self {
            DeltaTrigger::IncomingDamage => {}
            DeltaTrigger::Var(var) => {
                var_selector(var, ui);
            }
        }
    }
}

impl ToCstr for DeltaTrigger {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr()
    }
}
