use super::*;

#[derive(Default)]
pub struct Vfx {
    pub representation: Representation,
    pub anim: Anim,
}

impl Vfx {
    pub fn spawn(&self, t: &mut f32, world: &mut World) -> Result<(), ExpressionError> {
        let entity = world.spawn_empty().id();
        self.representation
            .clone()
            .unpack(entity, &mut world.commands());
        let context = Context::new_world(world).set_owner(entity).take();
        self.anim
            .get_changes(context)?
            .into_iter()
            .for_each(|c| c.apply(t, world));
        Ok(())
    }
}

impl Show for Vfx {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        self.representation.show(prefix, context, ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(None, &default(), ui);
        let mut r = self.anim.show_mut(Some("anim:"), ui);
        r |= self.representation.show_mut(Some("rep:"), ui);
        r
    }
}
