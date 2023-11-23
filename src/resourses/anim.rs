use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Anim {
    Sequence(Vec<Box<Anim>>),
    Run(Vec<Box<Anim>>),
    Change {
        var: VarName,
        value: Expression,
        #[serde(default)]
        t: f32,
        #[serde(default = "default_zero_f32_e")]
        duration: Expression,
        #[serde(default)]
        tween: Tween,
    },
}

fn default_zero_f32_e() -> Expression {
    Expression::Float(0.0)
}

impl Anim {
    pub fn apply(self, context: &Context, world: &mut World) -> Result<()> {
        match self {
            Anim::Sequence(list) => {
                for anim in list {
                    anim.apply(context, world)?;
                }
            }
            Anim::Run(list) => {
                start_batch(world);
                for anim in list {
                    to_batch_start(world);
                    anim.apply(context, world)?;
                }
                end_batch(world);
            }
            Anim::Change {
                var,
                t,
                duration,
                value,
                tween,
            } => {
                let entity = context.owner();
                let duration = duration.get_float(context, world)?;
                let value = value.get_value(context, world)?;
                let change = Change {
                    t,
                    duration,
                    tween,
                    value,
                };
                VarState::push_back(entity, var, change, world);
            }
        }
        Ok(())
    }
}
