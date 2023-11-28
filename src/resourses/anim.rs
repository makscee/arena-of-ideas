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
    pub fn apply(
        self,
        timeframe: Option<f32>,
        mut context: Context,
        world: &mut World,
    ) -> Result<()> {
        match self {
            Anim::Sequence(list) => {
                for anim in list {
                    anim.apply(timeframe, context.clone(), world)?;
                    context.incr_order();
                }
            }
            Anim::Run(list) => {
                for anim in list {
                    anim.apply(timeframe, context.clone(), world)?;
                }
            }
            Anim::Change {
                var,
                t,
                duration,
                value,
                tween,
            } => {
                let factor = timeframe.unwrap_or(1.0);
                let duration = duration.get_float(&context, world)? * factor;
                let value = value.get_value(&context, world)?;
                let change = Change {
                    t: factor * t,
                    duration,
                    tween,
                    value,
                };
                ActionCluster::get(world).push_change(
                    var,
                    change,
                    timeframe.unwrap_or_default(),
                    context,
                );
            }
        }
        Ok(())
    }
}
