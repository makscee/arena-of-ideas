use super::*;

#[derive(Default)]
pub struct Vfx {
    pub duration: f32,
    pub timeframe: f32,
    pub representation: Representation,
    pub anim: Anim,
}

impl Vfx {
    pub fn spawn(&self, t: &mut f32, world: &mut World) -> Result<Entity, ExpressionError> {
        let entity = world.spawn_empty().id();
        self.representation
            .clone()
            .unpack(entity, &mut world.commands());
        world.flush_commands();
        AnimChange::new_set(entity, VarName::visible, true.into()).apply(t, world);
        AnimChange::new_set(entity, VarName::visible, false.into())
            .apply(&mut (*t + self.duration), world);
        let context = Context::new_world(world).set_owner(entity).set_t(*t).take();
        self.anim
            .get_changes(context)?
            .into_iter()
            .for_each(|c| c.apply(t, world));
        AnimChange {
            entity,
            duration: 0.0,
            timeframe: 0.0,
            vars: [(VarName::t, 0.0.into())].into(),
        }
        .apply(t, world);
        AnimChange {
            entity,
            duration: self.duration,
            timeframe: self.timeframe,
            vars: [(VarName::t, 1.0.into())].into(),
        }
        .apply(t, world);
        Ok(entity)
    }
}

impl StringData for Vfx {
    fn inject_data(&mut self, data: &str) {}
    fn get_data(&self) -> String {
        let Vfx {
            duration,
            timeframe,
            representation,
            anim,
        } = self;
        ron::to_string(&(
            *duration,
            *timeframe,
            representation.get_data(),
            anim.get_data(),
        ))
        .unwrap()
    }
}
impl Show for Vfx {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.representation.show(prefix, context, ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        prefix.show(ui);
        let a = self.anim.show_mut(Some("anim:"), ui);
        self.representation.show_mut(Some("rep:"), ui) || a
    }
}
