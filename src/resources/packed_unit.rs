use super::*;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_description")]
    pub description: String,
    pub health: i32,
    #[serde(default)]
    pub damage: usize, // damage taken
    pub attack: usize,
    #[serde(default)]
    pub houses: HashSet<HouseName>,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub active_statuses: HashMap<String, i32>,
    pub shader: Option<Shader>,
}
fn default_name() -> String {
    "no_name".to_string()
}
fn default_description() -> String {
    "".to_string()
}

impl PackedUnit {
    pub fn pack(entity: legion::Entity, world: &legion::World, resources: &Resources) -> Self {
        let entry = world.entry_ref(entity).unwrap();
        let name = entry.get_component::<NameComponent>().unwrap().0.clone();
        let description = entry
            .get_component::<DescriptionComponent>()
            .unwrap()
            .text
            .clone();
        let health = entry.get_component::<HealthComponent>().unwrap().value;
        let damage = entry.get_component::<HealthComponent>().unwrap().damage;
        let attack = entry.get_component::<AttackComponent>().unwrap().value;
        let houses = HashSet::from_iter(
            entry
                .get_component::<HouseComponent>()
                .unwrap()
                .houses
                .iter()
                .cloned(),
        );
        let trigger = entry
            .get_component::<Trigger>()
            .cloned()
            .unwrap_or(Trigger::Noop);
        let active_statuses = resources
            .status_pool
            .active_statuses
            .get(&entity)
            .cloned()
            .unwrap_or_default();
        let shader = entry.get_component::<Shader>().ok().cloned();

        Self {
            name,
            description,
            health,
            damage,
            attack,
            houses,
            trigger,
            active_statuses,
            shader,
        }
    }

    pub fn unpack(
        &self,
        world: &mut legion::World,
        resources: &mut Resources,
        slot: usize,
        faction: Faction,
        position: Option<vec2<f32>>,
    ) -> legion::Entity {
        let entity = world.push((
            NameComponent::new(&self.name),
            DescriptionComponent::new(&self.description),
            AttackComponent::new(self.attack),
            HealthComponent::new(self.health, self.damage),
            HouseComponent::new(self.houses.clone()),
            AreaComponent::new(
                AreaType::Circle { radius: 1.0 },
                position.unwrap_or(vec2::ZERO),
            ),
            self.trigger.clone(),
            UnitComponent { slot, faction },
        ));
        resources.logger.log(
            &format!(
                "Unpacking unit {} new id: {:?} {:?}",
                self.name, entity, faction
            ),
            &LogContext::UnitCreation,
        );
        let world_entity = WorldSystem::get_context(world).owner;
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent::new(entity));
        entry.add_component(Context {
            owner: entity,
            target: entity,
            parent: Some(world_entity),
            vars: default(),
        });
        if let Some(shader) = &self.shader {
            entry.add_component(shader.clone());
        }

        resources
            .status_pool
            .active_statuses
            .insert(entity, self.active_statuses.clone());
        resources
            .input
            .listeners
            .insert(entity, Self::unit_drag_listener);
        ContextSystem::refresh_entity(entity, world, resources);
        Event::AfterBirth { owner: entity }.send(world, resources);
        entity
    }

    fn unit_drag_listener(
        entity: legion::Entity,
        resources: &mut Resources,
        world: &mut legion::World,
        event: InputEvent,
    ) {
        match event {
            InputEvent::Drag { .. } => {
                world.entry_mut(entity).ok().and_then(|mut entry| {
                    entry.get_component_mut::<AreaComponent>().unwrap().position =
                        resources.input.mouse_pos;
                    Some(())
                });
                resources.shop.drag_entity = Some(entity);
            }
            InputEvent::DragStop => {
                resources.shop.drop_entity = Some(entity);
            }
            _ => {}
        };
    }
}

impl FileWatcherLoader for PackedUnit {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        let unit = futures::executor::block_on(load_json(path)).unwrap();
        resources.hero_pool.insert(path.clone(), unit);
    }
}
