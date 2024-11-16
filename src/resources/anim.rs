use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, EnumIter, AsRefStr, PartialEq)]
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

impl Default for Anim {
    fn default() -> Self {
        Anim::Sequence(default())
    }
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

impl ToCstr for Anim {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr()
    }
}

impl ShowEditor for Anim {
    fn wrapper() -> Option<Self> {
        Some(Self::Run([default()].into()))
    }
    fn show_content(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Anim::Sequence(l) | Anim::Run(l) => {
                if Button::new("+").ui(ui).clicked() {
                    l.push(default());
                }
            }
            Anim::Change {
                var,
                value,
                t,
                duration,
                timeframe,
                tween,
            } => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        DragValue::new(t).prefix("t: ").ui(ui);
                        Selector::new("tween").ui_enum(tween, ui);
                        var_selector(var, ui);
                    });
                    value.show_node("value", context, world, ui);
                    duration.show_node("duration", context, world, ui);
                    timeframe.show_node("timeframe", context, world, ui);
                });
            }
            Anim::Sfx { sfx } => {
                Selector::new("sfx").ui_enum(sfx, ui);
            }
        }
    }
    fn get_variants() -> impl Iterator<Item = Self> {
        Self::iter()
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Anim::Sequence(l) | Anim::Run(l) => l.iter_mut().collect(),
            Anim::Change { .. } | Anim::Sfx { .. } => default(),
        }
    }
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Anim::Sequence(l) | Anim::Run(l) => {
                show_list_node(l, context, ui, world);
            }
            Anim::Change { .. } | Anim::Sfx { .. } => {}
        }
    }
}
