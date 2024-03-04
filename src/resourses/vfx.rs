use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, TypeUuid, TypePath)]
#[uuid = "4269cdf4-b418-4851-99ca-ce144438d2a3"]
pub struct Vfx {
    pub anim: Anim,
    pub representation: Representation,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub timeframe: f32,
    pub parent: Option<Entity>,
}

impl Vfx {
    pub fn unpack(self, world: &mut World) -> Result<()> {
        if SkipVisual::active(world) {
            return Ok(());
        }
        let entity = world.spawn_empty().id();
        self.representation.unpack(entity, world);
        if let Some(parent) = self.parent {
            world.entity_mut(entity).set_parent(parent);
        }
        self.state.attach(entity, world);
        let result = self.anim.apply(
            Context::new_named("vfx".to_owned())
                .set_owner(entity, world)
                .take(),
            world,
        );
        if self.timeframe > 0.0 {
            GameTimer::get().advance_insert(self.timeframe);
        }
        result
    }

    pub fn set_var(mut self, var: VarName, value: VarValue) -> Self {
        self.state.init(var, value);
        self
    }

    pub fn set_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn attach_context(mut self, context: &Context) -> Self {
        for (var, value) in context.get_all_vars() {
            self = self.set_var(var, value);
        }
        self
    }

    pub fn sort_history(mut self) -> Self {
        self.state.history.iter_mut().for_each(|(_, h)| {
            h.0.sort_by(|a, b| a.t.total_cmp(&b.t));
        });
        self
    }
}
