use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, AsRefStr, EnumIter)]
pub enum Effect {
    #[default]
    Noop,
    Damage,
    ChangeStatus(String),
    UseAbility(String),
}

impl Effect {
    pub fn invoke(&self, context: &mut Context, world: &mut World) -> Result<()> {
        debug!("Processing {:?}\n{:?}", self, context);
        let owner = context.owner();
        match self {
            Effect::Noop => {}
            Effect::Damage => {
                let target = context.get_target()?;
                let value = context
                    .get_var(VarName::Damage, world)
                    .unwrap_or(context.get_var(VarName::Pwr, world)?)
                    .get_int()?;
                if value > 0 {
                    VarState::get_mut(target, world).change_int(VarName::Dmg, value);
                }
            }
            Effect::ChangeStatus(name) => todo!(),
            Effect::UseAbility(name) => todo!(),
        }
        Ok(())
    }
}
