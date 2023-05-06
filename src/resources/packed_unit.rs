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
    pub house: Option<HouseName>,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub statuses: HashMap<String, i32>,
    pub shader: Option<Shader>,
    #[serde(default)]
    pub rank: u8,
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
        let state = entry.get_component::<ContextState>().unwrap();
        let name = state.name.clone();
        let description = state.vars.get_string(&VarName::Description);
        let health = state.vars.get_int(&VarName::HpValue);
        let damage = state.vars.get_int(&VarName::HpDamage) as usize;
        let attack = state.vars.get_int(&VarName::AttackValue) as usize;
        let house = state.vars.try_get_house();
        let trigger = entry
            .get_component::<Trigger>()
            .cloned()
            .unwrap_or(Trigger::Noop);
        let statuses = state.statuses.clone();
        let shader = entry.get_component::<Shader>().ok().cloned();
        let rank = state.vars.try_get_int(&VarName::Rank).unwrap_or_default() as u8;
        let result = Self {
            name,
            description,
            health,
            damage,
            attack,
            house,
            trigger,
            statuses,
            shader,
            rank,
        };
        resources.logger.log(
            || format!("Packing unit {} new id: {:?}", result, entity),
            &LogContext::UnitCreation,
        );
        result
    }

    pub fn unpack(
        &self,
        world: &mut legion::World,
        resources: &mut Resources,
        slot: usize,
        position: Option<vec2<f32>>,
        parent: Option<legion::Entity>,
    ) -> legion::Entity {
        let entity = world.push((self.trigger.clone(), UnitComponent {}));
        resources.logger.log(
            || format!("Unpacking unit {} new id: {:?} {:?}", self, entity, parent),
            &LogContext::UnitCreation,
        );
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent::new(entity));
        let mut state = ContextState::new(self.name.clone(), parent);
        state
            .vars
            .set_int(&VarName::AttackValue, self.attack as i32);
        state.vars.set_int(&VarName::HpValue, self.health as i32);
        state.vars.set_int(&VarName::HpDamage, self.damage as i32);
        state.vars.set_int(&VarName::Slot, slot as i32);
        state.vars.set_float(&VarName::Radius, 1.0);
        state
            .vars
            .set_vec2(&VarName::Position, position.unwrap_or(vec2::ZERO));
        if let Some(house) = self.house {
            state
                .vars
                .set_string(&VarName::House, 0, format!("{house:?}"));
            state
                .vars
                .set_color(&VarName::HouseColor, resources.house_pool.get_color(&house));
        }
        state
            .vars
            .set_string(&VarName::Description, 0, self.description.clone());
        state.statuses = self.statuses.clone();
        entry.add_component(state);

        entry.add_component(self.generate_shader(&resources.options));

        Event::AfterBirth { owner: entity }.send(world, resources);
        entity
    }

    pub fn generate_shader(&self, options: &Options) -> Shader {
        let mut shader = {
            if let Some(shader) = self.shader.as_ref() {
                let mut shader = shader.clone();
                shader.chain_after.push(options.shaders.unit.clone());
                shader
            } else {
                options.shaders.unit.clone()
            }
        };
        shader.chain_after.push(options.shaders.unit_card.clone());

        shader.set_string_ref(&VarName::Description.uniform(), self.description.clone(), 0);
        shader.chain_after.push(
            options
                .shaders
                .name
                .clone()
                .set_uniform("u_text", ShaderUniform::String((0, self.name.clone()))),
        );

        let hp_offset = options
            .shaders
            .stats
            .parameters
            .uniforms
            .try_get_vec2("u_offset")
            .unwrap()
            * vec2(-1.0, 1.0);
        let hp_card_offset = options
            .shaders
            .stats
            .parameters
            .uniforms
            .try_get_vec2("u_card_offset")
            .unwrap()
            * vec2(-1.0, 1.0);
        let hp_shader = options
            .shaders
            .stats
            .clone()
            .set_uniform("u_offset", ShaderUniform::Vec2(hp_offset))
            .set_uniform("u_card_offset", ShaderUniform::Vec2(hp_card_offset))
            .set_uniform("u_animate_on_damage", ShaderUniform::Float(1.0))
            .set_uniform("u_color", ShaderUniform::Color(options.colors.stats_health))
            .set_string("u_text", self.health.to_string(), 1)
            .set_int("u_value_modified", 0)
            .set_mapping("u_text", "u_hp_str")
            .set_mapping("u_value_modified", "u_hp_modified");
        shader.chain_after.push(hp_shader);

        let attack_shader = options
            .shaders
            .stats
            .clone()
            .set_uniform("u_color", ShaderUniform::Color(options.colors.stats_attack))
            .set_string("u_text", self.attack.to_string(), 1)
            .set_int("u_value_modified", 0)
            .set_mapping("u_text", "u_attack_str")
            .set_mapping("u_value_modified", "u_attack_modified");
        shader.chain_after.push(attack_shader);

        shader
    }
}

impl FileWatcherLoader for PackedUnit {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        debug!("Load unit {:?}", path);
        let unit = futures::executor::block_on(load_json(path)).unwrap();
        resources.hero_pool.insert(path.clone(), unit);
    }
}

impl fmt::Display for PackedUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{} {}/{}-{}]",
            self.name.as_str(),
            self.attack,
            self.health,
            self.damage
        )
    }
}
