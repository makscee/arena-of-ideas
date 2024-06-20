use super::*;

#[derive(Asset, Deserialize, Serialize, Debug, Clone, TypePath)]
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
    pub fn unpack(self, world: &mut World) -> Result<f32> {
        let entity = world.spawn_empty().id();
        self.representation.unpack(entity, world);
        if let Some(parent) = self.parent {
            world.entity_mut(entity).set_parent(parent);
        }
        self.state.attach(entity, world);
        let result = self.anim.apply(Context::new(entity), world);
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
}
