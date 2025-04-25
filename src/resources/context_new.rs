use std::any::type_name;

use super::*;

#[derive(Default, Debug)]
pub struct Context<'w> {
    pub t: Option<f32>,
    sources: Vec<ContextSource<'w>>,
    layers: Vec<ContextLayer>,
}

#[derive(Debug)]
pub enum ContextSource<'w> {
    Context(&'w Context<'w>),
    WorldRef(&'w World),
    WorldOwned(World),
    BattleSimulation(BattleSimulation),
}

#[derive(Debug, Clone)]
pub enum ContextLayer {
    Owner(Entity),
    Target(Entity),
    Caster(Entity),
    Var(VarName, VarValue),
}

impl<'w> Context<'w> {
    pub fn from_world_r(
        world: &mut World,
        f: impl FnOnce(&mut Self) -> Result<(), ExpressionError>,
    ) -> Result<(), ExpressionError> {
        let t = mem::take(world);
        let cs = ContextSource::WorldOwned(t);
        let mut context = Context {
            sources: [cs].into(),
            ..default()
        };
        let r = f(&mut context);
        let ContextSource::WorldOwned(t) = context.sources.remove(0) else {
            unreachable!()
        };
        *world = t;
        r
    }
    pub fn from_world(world: &mut World, f: impl FnOnce(&mut Self)) {
        Self::from_world_r(world, |context| {
            f(context);
            Ok(())
        })
        .log();
    }
    pub fn from_world_ref_r(
        world: &'w World,
        f: impl FnOnce(&mut Self) -> Result<(), ExpressionError>,
    ) -> Result<(), ExpressionError> {
        let mut context = Context {
            sources: [ContextSource::WorldRef(world)].into(),
            ..default()
        };
        f(&mut context)
    }
    pub fn from_battle_simulation_r(
        bs: &mut BattleSimulation,
        f: impl FnOnce(&mut Self) -> Result<(), ExpressionError>,
    ) -> Result<(), ExpressionError> {
        let mut context = Context {
            sources: [ContextSource::BattleSimulation(mem::take(bs))].into(),
            ..default()
        };
        let r = f(&mut context);
        let ContextSource::BattleSimulation(t) = context.sources.remove(0) else {
            unreachable!()
        };
        *bs = t;
        r
    }
    pub fn from_battle_simulation(bs: &mut BattleSimulation, f: impl FnOnce(&mut Self)) {
        Self::from_battle_simulation_r(bs, |context| {
            f(context);
            Ok(())
        })
        .log();
    }
    pub fn world_mut<'a>(&'a mut self) -> Result<&'a mut World, ExpressionError> {
        for s in &mut self.sources {
            let w = s.world_mut();
            if let Some(w) = w {
                return Ok(w);
            }
        }
        Err(ExpressionError::NotFound(
            "World not set for Context".into(),
        ))
    }
    pub fn world<'a>(&'a self) -> Result<&'a World, ExpressionError> {
        for s in &self.sources {
            let w = s.world();
            if let Some(w) = w {
                return Ok(w);
            }
        }
        Err(ExpressionError::NotFound(
            "World not set for Context".into(),
        ))
    }
    pub fn with_layer_r<T>(
        &mut self,
        layer: ContextLayer,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        self.with_layers_r([layer].into(), f)
    }
    pub fn with_layers_r<T>(
        &mut self,
        mut layers: Vec<ContextLayer>,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        let old_layers = self.layers.clone();
        self.layers.append(&mut layers);
        let r = f(self);
        self.layers = old_layers;
        r
    }
    pub fn with_layer(&mut self, layer: ContextLayer, f: impl FnOnce(&mut Self)) {
        self.with_layer_r(layer, |context| {
            f(context);
            Ok(())
        })
        .log();
    }
    pub fn with_layers(&mut self, layers: Vec<ContextLayer>, f: impl FnOnce(&mut Self)) {
        self.with_layers_r(layers, |context| {
            f(context);
            Ok(())
        })
        .log();
    }
    pub fn with_layer_ref_r<T>(
        &'w self,
        layer: ContextLayer,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        self.with_layers_ref_r([layer].into(), f)
    }
    pub fn with_layers_ref_r<T>(
        &'w self,
        mut layers: Vec<ContextLayer>,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        let mut all_layers = self.layers.clone();
        all_layers.append(&mut layers);
        let mut context = Self {
            t: self.t,
            sources: [ContextSource::Context(self)].into(),
            layers: all_layers,
        };
        f(&mut context)
    }
    pub fn with_layers_ref(&'w self, layers: Vec<ContextLayer>, f: impl FnOnce(&mut Self)) {
        self.with_layers_ref_r(layers, |context| {
            f(context);
            Ok(())
        })
        .log();
    }
    pub fn with_layer_ref(&'w self, layer: ContextLayer, f: impl FnOnce(&mut Self)) {
        self.with_layers_ref([layer].into(), f);
    }
    pub fn t(&self) -> Result<f32, ExpressionError> {
        self.t.to_custom_e("Context t not set")
    }
    pub fn t_mut(&mut self) -> Result<&mut f32, ExpressionError> {
        self.t.as_mut().to_custom_e("Context t not set")
    }
    pub fn id(&self, entity: Entity) -> Result<u64, ExpressionError> {
        self.world()?.entity_id(entity).to_e(entity)
    }
    pub fn entity(&self, id: u64) -> Result<Entity, ExpressionError> {
        self.world()?.id_entity(id).to_e(id)
    }
    pub fn parents(&self, id: u64) -> HashSet<u64> {
        self.world()
            .ok()
            .and_then(|w| w.children_parents_map().get(&id).cloned())
            .unwrap_or_default()
    }
    pub fn children(&self, id: u64) -> HashSet<u64> {
        self.world()
            .ok()
            .and_then(|w| w.parents_children_map().get(&id).cloned())
            .unwrap_or_default()
    }
    pub fn parents_recursive(&self, id: u64) -> HashSet<u64> {
        let mut result: HashSet<u64> = default();
        let mut q = VecDeque::from([id]);
        while let Some(id) = q.pop_front() {
            for parent in self.parents(id) {
                if !result.insert(parent) {
                    continue;
                }
                q.push_back(parent);
            }
        }
        result
    }
    pub fn children_recursive(&self, id: u64) -> HashSet<u64> {
        let mut result: HashSet<u64> = default();
        let mut q = VecDeque::from([id]);
        while let Some(id) = q.pop_front() {
            for child in self.children(id) {
                if !result.insert(child) {
                    continue;
                }
                q.push_back(child);
            }
        }
        result
    }
    pub fn first_parent<T: Component>(&self, entity: Entity) -> Result<&T, ExpressionError> {
        let id = self.id(entity)?;
        let mut checked: HashSet<u64> = default();
        let mut q = VecDeque::from([id]);
        while let Some(id) = q.pop_front() {
            for parent in self.parents(id) {
                if !checked.insert(parent) {
                    continue;
                }
                if let Ok(c) = self.get_by_id::<T>(parent) {
                    return Ok(c);
                }
                q.push_back(parent);
            }
        }
        Err(ExpressionError::NotFound(type_name::<T>().to_owned()))
    }
    pub fn first_child<T: Component>(&self, entity: Entity) -> Result<&T, ExpressionError> {
        let id = self.id(entity)?;
        let mut checked: HashSet<u64> = default();
        let mut q = VecDeque::from([id]);
        while let Some(id) = q.pop_front() {
            for child in self.children(id) {
                if !checked.insert(child) {
                    continue;
                }
                if let Ok(c) = self.get_by_id::<T>(child) {
                    return Ok(c);
                }
                q.push_back(child);
            }
        }
        Err(ExpressionError::NotFound(type_name::<T>().to_owned()))
    }
    pub fn add_parent_child(&mut self, parent: u64, child: u64) -> Result<(), ExpressionError> {
        self.world_mut()?.add_parent_child(parent, child);
        Ok(())
    }
    pub fn despawn(&mut self, entity: Entity) -> Result<(), ExpressionError> {
        self.world_mut()?.despawn_entity(entity);
        Ok(())
    }
    pub fn get<T: Component>(&self, entity: Entity) -> Result<&T, ExpressionError> {
        self.world()?.get::<T>(entity).to_e_not_found()
    }
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Result<Mut<T>, ExpressionError> {
        self.world_mut()?.get_mut::<T>(entity).to_e_not_found()
    }
    pub fn get_by_id<T: Component>(&self, id: u64) -> Result<&T, ExpressionError> {
        self.get::<T>(self.entity(id)?)
    }
    pub fn get_by_id_mut<T: Component>(&mut self, id: u64) -> Result<Mut<T>, ExpressionError> {
        self.get_mut::<T>(self.entity(id)?)
    }
    pub fn ids_to_entities(
        &self,
        ids: impl IntoIterator<Item = u64>,
    ) -> Result<Vec<Entity>, ExpressionError> {
        let m = self.world()?.id_to_entity_map();
        Ok(ids
            .into_iter()
            .filter_map(|id| m.get(&id).copied())
            .collect())
    }
    pub fn entities_to_ids(
        &self,
        entities: impl IntoIterator<Item = Entity>,
    ) -> Result<Vec<u64>, ExpressionError> {
        let m = self.world()?.entity_to_id_map();
        Ok(entities
            .into_iter()
            .filter_map(|entity| m.get(&entity).copied())
            .collect())
    }
    pub fn collect_components<T: Component>(
        &self,
        ids: impl IntoIterator<Item = u64>,
    ) -> Result<Vec<&T>, ExpressionError> {
        Ok(self
            .ids_to_entities(ids)?
            .into_iter()
            .filter_map(|entity| self.get::<T>(entity).ok())
            .collect())
    }
    pub fn collect_children_components<T: Component>(
        &self,
        id: u64,
    ) -> Result<Vec<&T>, ExpressionError> {
        self.collect_components(self.children(id))
    }
    pub fn collect_children_components_recursive<T: Component>(
        &self,
        id: u64,
    ) -> Result<Vec<&T>, ExpressionError> {
        self.collect_components(self.children_recursive(id))
    }

    pub fn battle_left_units(&self) -> Vec<Entity> {
        for s in &self.sources {
            match s {
                ContextSource::BattleSimulation(bs) => {
                    return bs.fusions_left.clone();
                }
                _ => {}
            }
        }
        default()
    }
    pub fn battle_right_units(&self) -> Vec<Entity> {
        for s in &self.sources {
            match s {
                ContextSource::BattleSimulation(bs) => {
                    return bs.fusions_right.clone();
                }
                _ => {}
            }
        }
        default()
    }
    pub fn battle_all_units(&self) -> Vec<Entity> {
        self.battle_left_units()
            .into_iter()
            .chain(self.battle_right_units())
            .collect()
    }
    pub fn battle_all_allies(&self, entity: Entity) -> Vec<Entity> {
        let left = self.battle_left_units();
        if left.contains(&entity) {
            return left;
        } else {
            let right = self.battle_right_units();
            if right.contains(&entity) {
                return right;
            }
        }
        default()
    }
    pub fn battle_all_enemies(&self, entity: Entity) -> Vec<Entity> {
        let left = self.battle_left_units();
        let right = self.battle_left_units();
        if left.contains(&entity) {
            return right;
        } else if right.contains(&entity) {
            return left;
        }

        default()
    }
    pub fn battle_offset_unit(&self, entity: Entity, offset: i32) -> Option<Entity> {
        let allies = self.battle_all_allies(entity);
        let pos = allies.iter().position(|e| *e == entity)?;
        allies.into_iter().enumerate().find_map(|(i, e)| {
            if i as i32 - pos as i32 == offset {
                Some(e)
            } else {
                None
            }
        })
    }

    pub fn owner_entity(&self) -> Result<Entity, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(e) = l.get_owner() {
                return Ok(e);
            }
        }
        Err(ExpressionError::NotFound("Owner not set".into()))
    }
    pub fn target_entity(&self) -> Result<Entity, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(e) = l.get_target() {
                return Ok(e);
            }
        }
        Err(ExpressionError::NotFound("Target not set".into()))
    }
    pub fn caster_entity(&self) -> Result<Entity, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(e) = l.get_caster() {
                return Ok(e);
            }
        }
        Err(ExpressionError::NotFound("Caster not set".into()))
    }
    pub fn collect_targets(&self) -> Vec<Entity> {
        self.layers.iter().filter_map(|l| l.get_target()).collect()
    }
    pub fn add_owner(&mut self, owner: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Owner(owner));
        self
    }
    pub fn add_target(&mut self, target: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Target(target));
        self
    }
    pub fn get_var(&self, var: VarName) -> Result<VarValue, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(v) = l.get_var(self, var) {
                return Ok(v);
            }
        }
        Err(ExpressionError::ValueNotFound(var))
    }
    pub fn sum_var(&self, var: VarName) -> Result<VarValue, ExpressionError> {
        let mut value = VarValue::default();
        for l in &self.layers {
            value = l.sum_var(self, var, value)?;
        }
        Ok(value)
    }
    pub fn get_value(&self) -> Result<VarValue, ExpressionError> {
        self.get_var(VarName::value)
    }
    pub fn set_var(&mut self, var: VarName, value: VarValue) -> &mut Self {
        self.layers.push(ContextLayer::Var(var, value));
        self
    }
    pub fn set_value_var(&mut self, value: VarValue) -> &mut Self {
        self.set_var(VarName::value, value);
        self
    }

    pub fn get_i32(&self, var: VarName) -> Result<i32, ExpressionError> {
        self.get_var(var)?.get_i32()
    }
    pub fn get_f32(&self, var: VarName) -> Result<f32, ExpressionError> {
        self.get_var(var)?.get_f32()
    }
    pub fn get_color(&self, var: VarName) -> Result<Color32, ExpressionError> {
        self.get_var(var)?.get_color()
    }
    pub fn color(&self, ui: &mut Ui) -> Color32 {
        self.get_color(VarName::color)
            .unwrap_or(ui.visuals().text_color())
    }
    pub fn get_string(&self, var: VarName) -> Result<String, ExpressionError> {
        self.get_var(var)?.get_string()
    }

    pub fn get_vars_layers(&self) -> HashMap<VarName, VarValue> {
        let mut result: HashMap<VarName, VarValue> = default();
        for l in self.layers.iter().rev() {
            match l {
                ContextLayer::Var(var, value) => {
                    result.insert(*var, value.clone());
                }
                _ => {}
            }
        }
        result
    }
}

impl ContextSource<'_> {
    fn world_mut(&mut self) -> Option<&mut World> {
        match self {
            ContextSource::WorldOwned(world) => Some(world),
            ContextSource::BattleSimulation(bs) => Some(&mut bs.world),
            _ => None,
        }
    }
    fn world(&self) -> Option<&World> {
        match self {
            ContextSource::WorldOwned(world) => Some(world),
            ContextSource::BattleSimulation(bs) => Some(&bs.world),
            ContextSource::WorldRef(world) => Some(*world),
            ContextSource::Context(context) => context.world().ok(),
        }
    }
}

impl ContextLayer {
    fn get_owner(&self) -> Option<Entity> {
        match self {
            ContextLayer::Owner(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_target(&self) -> Option<Entity> {
        match self {
            ContextLayer::Target(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_caster(&self) -> Option<Entity> {
        match self {
            ContextLayer::Caster(entity) => Some(*entity),
            _ => None,
        }
    }
    fn get_var(&self, context: &Context, var: VarName) -> Option<VarValue> {
        match self {
            ContextLayer::Var(vr, vl) => {
                if *vr == var {
                    Some(vl.clone())
                } else {
                    None
                }
            }
            ContextLayer::Owner(entity) => NodeState::find_var(context, var, *entity),
            _ => None,
        }
    }
    fn sum_var(
        &self,
        context: &Context,
        var: VarName,
        value: VarValue,
    ) -> Result<VarValue, ExpressionError> {
        match self {
            ContextLayer::Owner(entity) => NodeState::sum_var(context, var, *entity),
            ContextLayer::Var(vr, vl) => {
                if *vr == var {
                    value.add(vl)
                } else {
                    Ok(value)
                }
            }
            _ => Ok(value),
        }
    }
}
