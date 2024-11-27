use bevy::math::Quat;

use super::*;

#[derive(Debug, Clone, Default)]
pub struct Context {
    layers: Vec<ContextLayer>,
    t: f32,
}

impl Context {
    pub fn empty() -> Self {
        Self {
            t: gt().insert_head(),
            ..default()
        }
    }
    pub fn new(owner: Entity) -> Self {
        Self {
            layers: [ContextLayer::Owner(owner)].into(),
            t: gt().insert_head(),
        }
    }
    pub fn new_play(owner: Entity) -> Self {
        Self {
            layers: [ContextLayer::Owner(owner)].into(),
            t: gt().play_head(),
        }
    }
    pub fn set_owner(&mut self, entity: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Owner(entity));
        self
    }
    pub fn get_state<'a>(&self, world: &'a World) -> Option<&'a VarState> {
        self.layers.iter().rev().find_map(|l| l.get_state(world))
    }
    pub fn get_owner(&self) -> Result<Entity> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_owner())
            .with_context(|| format!("Failed to get owner"))
    }
    pub fn owner(&self) -> Entity {
        self.get_owner().expect("Owner not found")
    }
    pub fn set_target(&mut self, entity: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Target(entity));
        self
    }
    pub fn target(&self) -> Entity {
        self.get_target().expect("Target not found")
    }
    pub fn get_target(&self) -> Result<Entity> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_target())
            .with_context(|| format!("Failed to get target"))
    }
    pub fn set_caster(&mut self, entity: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Caster(entity));
        self
    }
    pub fn caster(&self) -> Entity {
        self.get_caster().expect("Caster not found")
    }
    pub fn get_caster(&self) -> Result<Entity> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_caster())
            .with_context(|| format!("Failed to get caster"))
    }
    pub fn get_value(&self, var: VarName, world: &World) -> Result<VarValue> {
        self.layers
            .iter()
            .rev()
            .find_map(|l| l.get_var(var, self.t, world))
            .with_context(|| format!("Failed to find var {var} {self:?}"))
    }
    pub fn get_charges(&self, world: &World) -> Result<i32> {
        self.get_value(VarName::Charges, world)?.get_int()
    }
    pub fn get_all_vars(&self) -> HashMap<VarName, VarValue> {
        let mut result: HashMap<VarName, VarValue> = default();
        for layer in self.layers.iter().rev() {
            if let ContextLayer::Var(var, value) = layer {
                result.insert(*var, value.clone());
            }
        }
        result
    }
    pub fn set_var(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.layers.push(ContextLayer::Var(var, value));
        self
    }
    pub fn set_event(&mut self, event: Event) -> &mut Self {
        self.layers.push(ContextLayer::Event(event));
        self
    }
    pub fn get_event(&self) -> Option<Event> {
        self.layers.iter().rev().find_map(|l| l.get_event())
    }
    pub fn set_effect(&mut self, effect: Cstr) -> &mut Self {
        self.layers.push(ContextLayer::Effect(effect));
        self
    }
    pub fn apply_transform(&self, vars: &[VarName], world: &mut World) {
        let entity = self.owner();
        let mut transform = world.entity_mut(entity).get::<Transform>().unwrap().clone();
        for var in vars {
            match var {
                VarName::Position => {
                    let position = VarState::try_get(entity, world)
                        .and_then(|s| s.get_value_at(*var, self.t))
                        .and_then(|v| v.get_vec2())
                        .unwrap_or_default();
                    transform.translation.x = position.x;
                    transform.translation.y = position.y;
                }
                VarName::Scale => {
                    let scale = self.get_vec2(*var, world).unwrap_or(Vec2::ONE);
                    transform.scale.x = scale.x;
                    transform.scale.y = scale.y;
                }
                VarName::Rotation => {
                    let rotation = self.get_float(*var, world).unwrap_or_default();
                    transform.rotation = Quat::from_rotation_z(rotation);
                }
                VarName::Offset => {
                    let position = VarState::try_get(entity, world)
                        .and_then(|s| s.get_value_at(*var, self.t))
                        .and_then(|v| v.get_vec2())
                        .unwrap_or_default();
                    transform.translation.x = position.x;
                    transform.translation.y = position.y;
                }
                _ => {}
            }
        }
        world.entity_mut(entity).insert(transform);
    }

    pub fn t(&self) -> f32 {
        self.t
    }
    pub fn t_to_insert(&mut self) {
        self.t = gt().insert_head();
    }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }

    pub fn all_active_statuses(&self, world: &World) -> HashMap<String, i32> {
        self.get_state(world)
            .map(|s| s.all_active_statuses_at(None, self.t))
            .unwrap_or_default()
    }

    pub fn get_birth(&self, world: &World) -> Result<f32> {
        Ok(self.get_state(world).context("State not found")?.birth())
    }
    pub fn get_bool(&self, var: VarName, world: &World) -> Result<bool> {
        self.get_value(var, world)?.get_bool()
    }
    pub fn get_int(&self, var: VarName, world: &World) -> Result<i32> {
        self.get_value(var, world)?.get_int()
    }
    pub fn get_float(&self, var: VarName, world: &World) -> Result<f32> {
        self.get_value(var, world)?.get_float()
    }
    pub fn get_vec2(&self, var: VarName, world: &World) -> Result<Vec2> {
        self.get_value(var, world)?.get_vec2()
    }
    pub fn get_string(&self, var: VarName, world: &World) -> Result<String> {
        self.get_value(var, world)?.get_string()
    }
    pub fn get_entity(&self, var: VarName, world: &World) -> Result<Entity> {
        self.get_value(var, world)?.get_entity()
    }

    pub fn detach(&mut self, world: &World) -> &mut Self {
        self.layers = mem::take(&mut self.layers)
            .into_iter()
            .flat_map(|l| match l {
                ContextLayer::Owner(entity) => [ContextLayer::Owner(entity)]
                    .into_iter()
                    .chain(
                        VarState::get(entity, world)
                            .all_values(self.t, world)
                            .into_iter()
                            .map(|(var, value)| ContextLayer::Var(var, value)),
                    )
                    .collect_vec(),
                _ => [l].into(),
            })
            .collect_vec();
        self
    }
    pub fn log(&self, main: Option<Cstr>) {
        if !is_dev_mode() {
            return;
        }
        let mut s = main.unwrap_or_default();
        for (i, layer) in self.layers.iter().enumerate() {
            s += &layer.cstr();
            if i != self.layers.len() - 1 {
                s += " -> ";
            }
        }
        s.debug()
    }
}

#[derive(Debug, Clone, AsRefStr, PartialEq)]
pub enum ContextLayer {
    Caster(Entity),
    Target(Entity),
    Owner(Entity),
    Var(VarName, VarValue),
    Event(Event),
    Effect(Cstr),
}

impl ContextLayer {
    fn get_state<'a>(&self, world: &'a World) -> Option<&'a VarState> {
        self.get_owner()
            .and_then(|e| VarState::try_get(e, world).ok())
    }
    fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_caster(&self) -> Option<Entity> {
        match self {
            ContextLayer::Caster(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_target(&self) -> Option<Entity> {
        match self {
            ContextLayer::Target(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_event(&self) -> Option<Event> {
        match self {
            ContextLayer::Event(e) => Some(e.clone()),
            _ => None,
        }
    }
    fn get_var(&self, var: VarName, t: f32, world: &World) -> Option<VarValue> {
        match self {
            ContextLayer::Owner(entity) => match VarState::try_get(*entity, world)
                .ok()
                .and_then(|state| state.get_value_at(var, t).ok())
            {
                Some(v) => Some(v),
                None => {
                    if let Some(entity) = entity.get_parent(world) {
                        ContextLayer::Owner(entity).get_var(var, t, world)
                    } else {
                        None
                    }
                }
            },
            ContextLayer::Var(v, value) => match var.eq(v) {
                true => Some(value.clone()),
                false => None,
            },
            _ => None,
        }
    }
}

impl ToCstr for ContextLayer {
    fn cstr(&self) -> Cstr {
        self.as_ref().to_owned()
            + &format!(
                "({})",
                match self {
                    ContextLayer::Caster(e) | ContextLayer::Target(e) | ContextLayer::Owner(e) => {
                        entity_name_with_id(*e)
                    }
                    ContextLayer::Var(var, value) => {
                        var.cstr() + " " + &value.cstr()
                    }
                    ContextLayer::Event(e) => e.cstr(),
                    ContextLayer::Effect(e) => e.clone(),
                }
            )
    }
}
