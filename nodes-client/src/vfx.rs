use super::*;

#[derive(Default)]
pub struct Vfx {
    pub duration: f32,
    pub timeframe: f32,
    pub anim: Anim,
}

impl Vfx {
    pub fn spawn(&self, t: &mut f32, world: &mut World) -> Result<f32, ExpressionError> {
        let context = Context::default().set_t(*t).take();
        self.anim.apply(t, context, world)
    }
}

impl StringData for Vfx {
    fn inject_data(&mut self, _: &str) {}
    fn get_data(&self) -> String {
        let Vfx {
            duration,
            timeframe,
            anim,
        } = self;
        ron::to_string(&(*duration, *timeframe, anim.get_data())).unwrap()
    }
}
impl Show for Vfx {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.anim.show(None, context, ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(ui);
        self.anim.show_mut(Some("anim:"), ui)
    }
}
