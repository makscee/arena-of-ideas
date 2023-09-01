use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Effect {
    Damage { value: Option<Expression> },
}

impl Effect {
    pub fn process(self, mut context: Context, world: &mut World) -> Result<()> {
        match self {
            Effect::Damage { value } => {
                if let Some(value) = value {
                    let target = context.get_target().context("Target not found")?;
                    let value = value.get_int(&context, world)?;
                    debug!("Damage {value} {target:?}");

                    let state = world
                        .get_mut::<VarState>(target)
                        .context("Target state not found")?;
                    let new_hp = state.get_int(VarName::Hp)? - value;
                    VarState::push_back(
                        target,
                        VarName::Hp,
                        Change::new(VarValue::Int(new_hp)),
                        world,
                    );
                }
            }
        }
        Ok(())
    }
}
