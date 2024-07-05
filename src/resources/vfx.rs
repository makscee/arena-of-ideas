use super::*;

#[derive(Asset, Deserialize, Serialize, Debug, Clone, TypePath)]
pub struct Vfx {
    pub anim: Anim,
    pub representation: Representation,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub timeframe: f32,
    #[serde(default)]
    pub duration: Option<f32>,
    pub parent: Option<Entity>,
}

impl Vfx {
    pub fn get(name: &str, world: &World) -> Self {
        GameAssets::get(world)
            .vfxs
            .get(name)
            .with_context(|| format!("Vfx {name} not loaded"))
            .unwrap()
            .clone()
    }
    pub fn unpack(self, world: &mut World) -> Result<f32> {
        let entity = world.spawn_empty().id();
        self.representation.unpack(entity, world);
        if let Some(parent) = self.parent {
            world.entity_mut(entity).set_parent(parent);
        }
        self.state.attach(entity, 0, world);
        let result = self.anim.apply(Context::new(entity), world);
        if let Some(duration) = self.duration {
            let mut state = VarState::get_mut(entity, world);
            state.init(VarName::Visible, true.into());
            state.push_change(
                VarName::Visible,
                default(),
                VarChange {
                    t: duration,
                    ..default()
                },
            );
        }
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
