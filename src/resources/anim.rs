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
        #[serde(default = "default_zero_f32_e")]
        timeframe: Expression,
        #[serde(default)]
        tween: Tween,
    },
}

fn default_zero_f32_e() -> Expression {
    Expression::Float(0.0)
}

impl Anim {
    pub fn apply(self, context: Context, world: &mut World) -> Result<()> {
        match self {
            Anim::Sequence(list) => {
                for anim in list {
                    anim.apply(context.clone(), world)?;
                }
            }
            Anim::Run(list) => {
                for anim in list {
                    GameTimer::get().start_batch();
                    anim.apply(context.clone(), world)?;
                    GameTimer::get().to_batch_start().end_batch();
                }
            }
            Anim::Change {
                var,
                t,
                duration,
                timeframe,
                value,
                tween,
            } => {
                let duration = duration.get_float(&context, world)?;
                let timeframe = timeframe.get_float(&context, world)?;
                let value = value.get_value(&context, world)?;
                let change = VarChange {
                    t,
                    duration,
                    timeframe,
                    tween,
                    value,
                };
                VarState::get_mut(context.owner(), world).push_back(var, change);
                GameTimer::get().advance_insert(timeframe);
            }
        }
        Ok(())
    }
}
