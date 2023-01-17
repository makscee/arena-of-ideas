use super::*;
use strfmt::strfmt;

// Display text message
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MessageEffect {
    pub text: String,
    pub color: Option<Rgba<f32>>,
    pub position: Option<Position>,
}

impl MessageEffect {
    pub fn create(text: String, color: Option<Rgba<f32>>, position: Position) -> LogicEffect {
        LogicEffect::Message(Box::new(MessageEffect {
            text,
            color,
            position: Some(position),
        }))
    }
}

impl EffectContainer for MessageEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {}
}

impl EffectImpl for MessageEffect {
    fn process(self: Box<Self>, mut context: LogicEffectContext, logic: &mut Logic) {
        // todo: reimplement
        // if let Some(color) = &self.color {
        //     context.color = color.clone();
        // }
        // let position = match self.position {
        //     Some(position) => position,
        //     None => logic.model.get_who(Who::Target, &context).position,
        // };
        // let text = strfmt(&self.text, &context.vars).unwrap_or(self.text);
        // logic.model.render_model.add_text(
        //     position,
        //     text,
        //     context.color,
        //     crate::render::TextType::Message,
        // );
    }
}
