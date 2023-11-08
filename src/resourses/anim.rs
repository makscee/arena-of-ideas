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
        // debug!("{}\n{context:#?}", "Apply animation".purple());
        match self {
            Anim::Sequence(list) => {
                for anim in list {
                    anim.apply(context, world)?;
                }
            }
            Anim::Run(list) => {
                GameTimer::get_mut(world).start_batch();
                for anim in list {
                    GameTimer::get_mut(world).head_to_batch_start();
                    anim.apply(context, world)?;
                }
                GameTimer::get_mut(world).end_batch();
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
