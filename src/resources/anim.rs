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
    Sfx {
        sfx: SoundEffect,
    },
}

fn default_zero_f32_e() -> Expression {
    Expression::Value(VarValue::Float(0.0))
}

impl Anim {
    pub fn apply(self, context: Context, world: &mut World) -> Result<f32> {
        let mut head_shift = 0.0;
        match self {
            Anim::Sequence(list) => {
                for anim in list {
                    head_shift += anim.apply(context.clone(), world)?;
                }
            }
            Anim::Run(list) => {
                for anim in list {
                    head_shift = anim.apply(context.clone(), world)?.max(head_shift);
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
                head_shift = timeframe.get_float(&context, world)?;
                let value = value.get_value(&context, world)?;
                let change = VarChange {
                    t,
                    duration,
                    timeframe: head_shift,
                    tween,
                    value,
                };
                VarState::get_mut(context.owner(), world).push_change(var, default(), change);
            }
            Anim::Sfx { sfx } => {
                ActionPlugin::register_sound_effect(sfx, world);
            }
        }
        Ok(head_shift)
    }
}
