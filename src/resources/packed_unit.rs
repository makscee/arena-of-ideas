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
    pub statuses: Vec<(String, i32)>,
    pub shader: Option<ShaderChain>,
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
        let statuses = StatusSystem::pack_state_into_vec(state);
        let shader = entry.get_component::<ShaderChain>().ok().cloned();
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
            || {
                format!(
                    "Unpacking unit {} new id: {:?} p:{:?}",
                    self, entity, parent
                )
            },
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
        state.vars.set_int(&VarName::Rank, self.rank as i32);
        state
            .vars
            .set_vec2(&VarName::Position, position.unwrap_or(vec2::ZERO));
        let house_color = self.house_color(resources);
        state.vars.set_color(&VarName::HouseColor, house_color);
        if let Some(house) = self.house {
            state
                .vars
                .set_string(&VarName::House, 0, format!("{house:?}"));
        }
        state
            .vars
            .set_string(&VarName::Description, 0, self.description.clone());
        StatusSystem::unpack_into_state(&mut state, &self.statuses);
        entry.add_component(state);

        entry.add_component(self.generate_shader(house_color, &resources.options));

        Event::AfterBirth { owner: entity }.send(world, resources);
        entity
    }

    pub fn house_color(&self, resources: &Resources) -> Rgba<f32> {
        if let Some(house) = self.house {
            HousePool::get_color(&house, resources)
        } else {
            Rgba::MAGENTA
        }
    }

    pub fn generate_shader(&self, house_color: Rgba<f32>, options: &Options) -> ShaderChain {
        let mut shader = options.shaders.unit.clone();
        if let Some(self_shader) = self.shader.as_ref() {
            shader.before.push(self_shader.clone());
        }
        shader.insert_color_ref("u_house_color".to_owned(), house_color);
        shader.after.push(options.shaders.unit_card.clone());

        shader.insert_string_ref("u_description".to_owned(), self.description.clone(), 0);
        shader.insert_float_ref("u_rank_1".to_owned(), (self.rank > 0) as i32 as f32);
        shader.insert_float_ref("u_rank_2".to_owned(), (self.rank > 1) as i32 as f32);
        shader.insert_float_ref("u_rank_3".to_owned(), (self.rank > 2) as i32 as f32);

        let hp_offset = options
            .shaders
            .stats
            .middle
            .parameters
            .uniforms
            .try_get_vec2("u_offset")
            .unwrap()
            * vec2(-1.0, 1.0);
        let hp_card_offset = options
            .shaders
            .stats
            .middle
            .parameters
            .uniforms
            .try_get_vec2("u_card_offset")
            .unwrap()
            * vec2(-1.0, 1.0);
        let hp_shader = options
            .shaders
            .stats
            .clone()
            .insert_uniform("u_offset".to_owned(), ShaderUniform::Vec2(hp_offset))
            .insert_uniform(
                "u_card_offset".to_owned(),
                ShaderUniform::Vec2(hp_card_offset),
            )
            .insert_uniform(
                "u_color".to_owned(),
                ShaderUniform::Color(options.colors.stat_hp),
            )
            .insert_string("u_text".to_owned(), self.health.to_string(), 1)
            .map_key_to_key("u_text", "u_hp_str")
            .map_key_to_key("u_text_extra_size", "u_damage_taken")
            .map_key_to_key("u_text_color", "u_hp_color");
        shader.after.push(hp_shader);

        let attack_shader = options
            .shaders
            .stats
            .clone()
            .insert_uniform(
                "u_color".to_owned(),
                ShaderUniform::Color(options.colors.stat_atk),
            )
            .insert_string("u_text".to_owned(), self.attack.to_string(), 1)
            .map_key_to_key("u_text", "u_attack_str")
            .map_key_to_key("u_text_color", "u_attack_color");
        shader.after.push(attack_shader);
        shader
            .after
            .push(options.shaders.name.clone().insert_uniform(
                "u_text".to_owned(),
                ShaderUniform::String((1, self.name.clone())),
            ));

        shader
    }

    pub fn get_ui_shader(
        &self,
        faction: Faction,
        set_card: bool,
        resources: &Resources,
    ) -> ShaderChain {
        let house_color = self.house_color(resources);
        let mut shader = self.generate_shader(house_color, &resources.options);
        shader
            .middle
            .parameters
            .uniforms
            .insert_color_ref(
                "u_faction_color".to_owned(),
                faction.color(&resources.options),
            )
            .insert_vec2_ref("u_align".to_owned(), vec2::ZERO);
        if set_card {
            shader.middle.insert_float_ref("u_card".to_owned(), 1.0);
        }
        if let Some(house) = self.house {
            shader.middle.parameters.uniforms.insert_color_ref(
                "u_house_color".to_owned(),
                HousePool::get_color(&house, resources),
            );
        }
        let mut definitions = UnitSystem::extract_definitions(&self.description, resources);
        definitions.extend(self.statuses.iter().map(|(name, _)| name.clone()));
        StatusSystem::add_active_statuses_hint(&mut shader.middle, &self.statuses, resources);
        Definitions::add_hints(&mut shader.middle, definitions, resources);
        shader.middle.entity = Some(new_entity());
        shader
    }
}

impl FileWatcherLoader for PackedUnit {
    fn load(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::load));
        debug!("Load unit {:?}", path);
        let unit = futures::executor::block_on(load_json(path)).unwrap();
        HeroPool::insert(path.clone(), unit, resources);
    }
}

impl fmt::Display for PackedUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{} {}/{}-{} r:{} h:{}]",
            self.name.as_str(),
            self.attack,
            self.health,
            self.damage,
            self.rank,
            self.house.map(|x| x.to_string()).unwrap_or("_".to_owned()),
        )
    }
}
