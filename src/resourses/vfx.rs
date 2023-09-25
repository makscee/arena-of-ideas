use super::*;

#[derive(Deserialize, Debug, Clone, TypeUuid, TypePath)]
#[uuid = "4269cdf4-b418-4851-99ca-ce144438d2a3"]
pub struct Vfx {
    pub anim: Anim,
    pub representation: Representation,
    #[serde(default)]
    pub state: VarState,
}

impl Vfx {
    pub fn unpack(self, world: &mut World) -> Result<()> {
        let entity = self.representation.unpack(None, None, world);
        self.state.attach(entity, world);
        let t = get_insert_t(world);
        let result = self.anim.apply(
            &Context::new_named("vfx".to_owned())
                .set_owner(entity, world)
                .take(),
            world,
        );
        GameTimer::get_mut(world).set_insert_t(t);
        result
    }

    pub fn set_var(mut self, var: VarName, value: VarValue) -> Self {
        self.state.init(var, value);
        self
    }
}
