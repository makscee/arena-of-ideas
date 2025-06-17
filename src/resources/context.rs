use super::*;

#[derive(Debug)]
pub struct Context<'w> {
    pub t: Option<f32>,
    rng: ChaCha8Rng,
    sources: Vec<ContextSource<'w>>,
    layers: Vec<ContextLayer>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            t: None,
            rng: rng_seeded(now_micros() as u64),
            sources: default(),
            layers: default(),
        }
    }
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
    pub fn from_world_r<T>(
        world: &mut World,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        let mut t = mem::take(world);
        t.init_links();
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
    pub fn from_world_ref_r<T>(
        world: &'w World,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        let mut context = Context {
            sources: [ContextSource::WorldRef(world)].into(),
            ..default()
        };
        f(&mut context)
    }
    pub fn from_battle_simulation_r<T>(
        bs: &mut BattleSimulation,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        let t = bs.duration;
        let mut context = Context {
            t: Some(t),
            sources: [ContextSource::BattleSimulation(mem::take(bs))].into(),
            rng: rng_seeded(bs.seed),
            layers: default(),
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
        Err(ExpressionErrorVariants::NotFound("World not set for Context".into()).into())
    }
    pub fn world<'a>(&'a self) -> Result<&'a World, ExpressionError> {
        for s in &self.sources {
            let w = s.world();
            if let Some(w) = w {
                return Ok(w);
            }
        }
        Err(ExpressionErrorVariants::NotFound("World not set for Context".into()).into())
    }
    pub fn rng<'a>(&'a mut self) -> &'a mut impl Rng {
        &mut self.rng
    }
    pub fn battle_simulation_mut<'a>(
        &'a mut self,
    ) -> Result<&'a mut BattleSimulation, ExpressionError> {
        for s in &mut self.sources {
            let bs = s.battle_simulation_mut();
            if let Some(bs) = bs {
                return Ok(bs);
            }
        }
        Err(ExpressionErrorVariants::NotFound("BattleSimulation not set for Context".into()).into())
    }
    pub fn battle_simulation<'a>(&'a self) -> Result<&'a BattleSimulation, ExpressionError> {
        for s in &self.sources {
            let bs = s.battle_simulation();
            if let Some(bs) = bs {
                return Ok(bs);
            }
        }
        Err(ExpressionErrorVariants::NotFound("BattleSimulation not set for Context".into()).into())
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
            rng: self.rng.clone(),
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
    pub fn with_owner<T>(
        &mut self,
        entity: Entity,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        self.with_layer_r(ContextLayer::Owner(entity), f)
    }
    pub fn with_owner_ref<T>(
        &'w self,
        entity: Entity,
        f: impl FnOnce(&mut Self) -> Result<T, ExpressionError>,
    ) -> Result<T, ExpressionError> {
        self.with_layer_ref_r(ContextLayer::Owner(entity), f)
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
    pub fn parents_entity(&self, entity: Entity) -> Result<Vec<Entity>, ExpressionError> {
        let id = entity.id(self)?;
        self.ids_to_entities(self.parents(id))
    }
    pub fn children_entity(&self, entity: Entity) -> Result<Vec<Entity>, ExpressionError> {
        let id = entity.id(self)?;
        self.ids_to_entities(self.children(id))
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
    pub fn first_parent<T: Component>(&self, id: u64) -> Result<&T, ExpressionError> {
        for parent in self.parents(id) {
            let c = self.get_by_id::<T>(parent);
            if c.is_ok() {
                return c;
            }
        }
        Err(ExpressionErrorVariants::NotFound(type_name_short::<T>().to_owned()).into())
    }
    pub fn first_child<T: Component>(&self, id: u64) -> Result<&T, ExpressionError> {
        for child in self.children(id) {
            let c = self.get_by_id::<T>(child);
            if c.is_ok() {
                return c;
            }
        }
        Err(ExpressionErrorVariants::NotFound(type_name_short::<T>().to_owned()).into())
    }
    pub fn first_parent_recursive<T: Component>(&self, id: u64) -> Result<&T, ExpressionError> {
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
        Err(ExpressionErrorVariants::NotFound(type_name_short::<T>().to_owned()).into())
    }
    pub fn first_child_recursive<T: Component>(&self, id: u64) -> Result<&T, ExpressionError> {
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
        Err(ExpressionErrorVariants::NotFound(type_name_short::<T>().to_owned()).into())
    }
    pub fn link_id_entity(&mut self, id: u64, entity: Entity) -> Result<(), ExpressionError> {
        self.world_mut()?.link_id_entity(id, entity);
        Ok(())
    }
    pub fn link_parent_child(&mut self, parent: u64, child: u64) -> Result<(), ExpressionError> {
        self.world_mut()?.link_parent_child(parent, child);
        Ok(())
    }
    pub fn unlink_parent_child(
        &mut self,
        parent: u64,
        child: u64,
    ) -> Result<bool, ExpressionError> {
        Ok(self.world_mut()?.unlink_parent_child(parent, child))
    }
    pub fn link_parent_child_entity(
        &mut self,
        parent: Entity,
        child: Entity,
    ) -> Result<(), ExpressionError> {
        self.link_parent_child(self.id(parent)?, self.id(child)?)
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
    pub fn collect_parents_components<T: Component>(
        &self,
        id: u64,
    ) -> Result<Vec<&T>, ExpressionError> {
        self.collect_components(self.parents(id))
    }
    pub fn collect_children_components<T: Component>(
        &self,
        id: u64,
    ) -> Result<Vec<&T>, ExpressionError> {
        self.collect_components(self.children(id))
    }
    pub fn collect_parents_components_recursive<T: Component>(
        &self,
        id: u64,
    ) -> Result<Vec<&T>, ExpressionError> {
        self.collect_components(self.parents_recursive(id))
    }
    pub fn collect_children_components_recursive<T: Component>(
        &self,
        id: u64,
    ) -> Result<Vec<&T>, ExpressionError> {
        self.collect_components(self.children_recursive(id))
    }

    pub fn owner_entity(&self) -> Result<Entity, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(e) = l.get_owner() {
                return Ok(e);
            }
        }
        Err(ExpressionErrorVariants::NotFound("Owner not set".into()).into())
    }
    pub fn target_entity(&self) -> Result<Entity, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(e) = l.get_target() {
                return Ok(e);
            }
        }
        Err(ExpressionErrorVariants::NotFound("Target not set".into()).into())
    }
    pub fn caster_entity(&self) -> Result<Entity, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(e) = l.get_caster() {
                return Ok(e);
            }
        }
        Err(ExpressionErrorVariants::NotFound("Caster not set".into()).into())
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
    pub fn add_caster(&mut self, caster: Entity) -> &mut Self {
        self.layers.push(ContextLayer::Caster(caster));
        self
    }
    pub fn get_var(&self, var: VarName) -> Result<VarValue, ExpressionError> {
        for l in self.layers.iter().rev() {
            if let Some(v) = l.get_var(self, var) {
                return Ok(v);
            }
        }
        Err(ExpressionErrorVariants::ValueNotFound(var).into())
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
    pub fn get_vec2(&self, var: VarName) -> Result<Vec2, ExpressionError> {
        self.get_var(var)?.get_vec2()
    }
    pub fn get_bool(&self, var: VarName) -> Result<bool, ExpressionError> {
        self.get_var(var)?.get_bool()
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
    fn battle_simulation_mut(&mut self) -> Option<&mut BattleSimulation> {
        match self {
            ContextSource::BattleSimulation(bs) => Some(bs),
            _ => None,
        }
    }
    fn battle_simulation(&self) -> Option<&BattleSimulation> {
        match self {
            ContextSource::Context(context) => context.battle_simulation().ok(),
            ContextSource::BattleSimulation(bs) => Some(bs),
            _ => None,
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
