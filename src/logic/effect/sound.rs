use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SoundEffect {
    pub name: String,
    #[serde(default)]
    pub r#loop: bool,
}

impl EffectContainer for SoundEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {}
}

impl EffectImpl for SoundEffect {
    fn process(self: Box<Self>, context: LogicEffectContext, logic: &mut logic::Logic) {
        // todo: reimplement
        // let effect = *self;
        // logic.sound_controller.play_sound(effect.name.clone());
    }
}
