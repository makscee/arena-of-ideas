use colored::CustomColor;

use crate::module_bindings::TableUnit;

use super::*;

#[derive(AssetCollection, Resource, Debug)]
pub struct Pools {
    #[asset(key = "pool.heroes", collection(typed, mapped))]
    heroes_handles: HashMap<String, Handle<PackedUnit>>,
    pub heroes: HashMap<String, PackedUnit>,
    #[asset(key = "pool.houses", collection(typed, mapped))]
    houses_handles: HashMap<String, Handle<House>>,
    pub houses: HashMap<String, House>,
    pub statuses: HashMap<String, PackedStatus>,
    pub abilities: HashMap<String, Ability>,
    pub summons: HashMap<String, PackedUnit>,
    pub default_ability_states: HashMap<String, VarState>,
    #[asset(key = "pool.vfx", collection(typed, mapped))]
    vfx_handles: HashMap<String, Handle<Vfx>>,
    pub vfx: HashMap<String, Vfx>,
    pub colors: HashMap<String, Color>,
    pub only_local_cache: bool,
}

impl Pools {
    pub fn get(world: &World) -> &Self {
        world.get_resource::<Pools>().unwrap()
    }
    pub fn try_get(world: &World) -> Option<&Self> {
        world.get_resource::<Pools>()
    }
    pub fn get_mut(world: &mut World) -> Mut<Self> {
        world.get_resource_mut::<Pools>().unwrap()
    }

    pub fn get_status<'a>(name: &str, world: &'a World) -> Option<&'a PackedStatus> {
        Self::get(world).statuses.get(name)
    }
    pub fn get_ability<'a>(name: &str, world: &'a World) -> Option<&'a Ability> {
        Self::get(world).abilities.get(name)
    }
    pub fn get_summon<'a>(name: &str, world: &'a World) -> Option<&'a PackedUnit> {
        Self::get(world).summons.get(name)
    }
    pub fn get_vfx(name: &str, world: &World) -> Vfx {
        let name = if name.ends_with(".ron") {
            name.to_owned()
        } else {
            format!("ron/vfx/{name}.vfx.ron")
        };
        Self::get(world).vfx.get(&name).unwrap().clone()
    }
    pub fn get_ability_house<'a>(name: &str, world: &'a World) -> Option<&'a House> {
        Self::get(world)
            .houses
            .iter()
            .find(|(_, h)| h.abilities.iter().any(|a| a.name.eq(name)))
            .map(|(_, h)| h)
    }
    pub fn get_status_house<'a>(name: &str, world: &'a World) -> Option<&'a House> {
        Self::get(world)
            .houses
            .iter()
            .find(|(_, h)| h.statuses.iter().any(|s| s.name.eq(name)))
            .map(|(_, h)| h)
    }
    pub fn get_summon_house<'a>(name: &str, world: &'a World) -> Option<&'a House> {
        Self::get(world)
            .houses
            .iter()
            .find(|(_, h)| h.summons.iter().any(|s| s.name.eq(name)))
            .map(|(_, h)| h)
    }
    pub fn get_house_color(name: &str, world: &World) -> Option<Color> {
        Self::try_get(world).and_then(|p| p.house_color(name))
    }
    pub fn house_color(&self, name: &str) -> Option<Color> {
        self.houses.get(name).map(|h| h.color.clone().into())
    }
    pub fn get_color_by_name(name: &str, world: &World) -> Result<Color> {
        Self::get(world)
            .color_by_name(name)
            .with_context(|| format!("Color not found for {name}"))
    }
    pub fn color_by_name(&self, name: &str) -> Option<Color> {
        self.colors.get(name).cloned()
    }
    pub fn get_default_ability_state<'a>(name: &str, world: &'a World) -> Option<&'a VarState> {
        Self::try_get(world)?.default_ability_states.get(name)
    }
}

pub struct PoolsPlugin;

impl PoolsPlugin {
    pub fn setup(world: &mut World) {
        Self::setup_houses(world);
        Self::setup_statuses(world);
        Self::setup_abilities(world);
        Self::setup_summons(world);
        Self::setup_heroes(world);
        Self::setup_vfx(world);
    }

    pub fn setup_houses(world: &mut World) {
        let houses = HashMap::from_iter(
            world
                .get_resource::<Pools>()
                .unwrap()
                .houses_handles
                .values()
                .map(|handle| {
                    let house = world
                        .get_resource::<Assets<House>>()
                        .unwrap()
                        .get(handle)
                        .unwrap()
                        .clone();
                    (house.name.to_owned(), house)
                }),
        );
        debug!("Setup {} houses", houses.len());
        let mut pools = Pools::get_mut(world);
        for house in houses.values() {
            for (ability, data) in house.defaults.iter() {
                pools
                    .default_ability_states
                    .insert(ability.to_owned(), data.clone().into());
            }
            let color: Color = house.color.clone().into();
            for ability in house.abilities.iter() {
                pools.colors.insert(ability.name.clone(), color.clone());
            }
            for status in house.statuses.iter() {
                pools.colors.insert(status.name.clone(), color.clone());
            }
            for summon in house.summons.iter() {
                pools.colors.insert(summon.name.clone(), color.clone());
            }
        }
        pools.houses = houses;
    }

    pub fn setup_vfx(world: &mut World) {
        let vfx = HashMap::from_iter(
            world
                .get_resource::<Pools>()
                .unwrap()
                .vfx_handles
                .iter()
                .map(|(path, handle)| {
                    let vfx = world
                        .get_resource::<Assets<Vfx>>()
                        .unwrap()
                        .get(handle)
                        .unwrap()
                        .clone()
                        .sort_history();

                    (path.to_owned(), vfx)
                }),
        );
        debug!("Setup {} vfx", vfx.len());
        world.get_resource_mut::<Pools>().unwrap().vfx = vfx;
    }

    pub fn setup_statuses(world: &mut World) {
        let statuses = Pools::get(world)
            .houses
            .iter()
            .flat_map(|(_, h)| {
                let mut statuses = h.statuses.clone();
                for status in statuses.iter_mut() {
                    status
                        .state
                        .init(VarName::Color, VarValue::Color(h.color.clone().into()));
                }
                statuses
            })
            .collect_vec();
        let pool = &mut Pools::get_mut(world).statuses;
        debug!("Setup {} statuses", statuses.len());
        for (key, value) in statuses.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate status name: {key}")
            }
        }
    }

    pub fn setup_abilities(world: &mut World) {
        let abilities = Pools::get(world)
            .houses
            .iter()
            .flat_map(|(_, h)| h.abilities.clone())
            .collect_vec();
        let pool = &mut Pools::get_mut(world).abilities;
        debug!("Setup {} abilities", abilities.len());
        for (key, value) in abilities.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate ability name: {key}")
            }
        }
    }

    pub fn setup_summons(world: &mut World) {
        let summons = Pools::get(world)
            .houses
            .iter()
            .flat_map(|(_, h)| h.summons.clone())
            .collect_vec();
        let pool = &mut Pools::get_mut(world).summons;
        debug!("Setup {} summons", summons.len());
        for (key, value) in summons.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate summon name: {key}")
            }
        }
    }

    pub fn setup_heroes(world: &mut World) {
        let heroes = world
            .get_resource::<Pools>()
            .unwrap()
            .heroes_handles
            .values()
            .map(|handle| {
                world
                    .get_resource::<Assets<PackedUnit>>()
                    .unwrap()
                    .get(handle)
                    .unwrap()
                    .clone()
            })
            .collect_vec();
        let pool = &mut Pools::get_mut(world).heroes;
        debug!("Setup {} heroes", heroes.len());
        for (key, value) in heroes.into_iter().map(|s| (s.name.clone(), s)) {
            if pool.insert(key.clone(), value).is_some() {
                panic!("Duplicate hero name: {key}")
            }
        }
    }

    pub fn cache_server_pools(world: &mut World) {
        if module_bindings::House::count() == 0 {
            error!("Server assets are not synced");
            return;
        }
        let mut pools = Pools::get_mut(world);
        if pools.only_local_cache {
            debug!("Server cache disabled");
            return;
        }
        debug!("Cache server pools start");
        pools.heroes.clear();
        pools.houses.clear();
        pools.abilities.clear();
        pools.statuses.clear();
        pools.vfx.clear();
        for unit in TableUnit::iter() {
            pools.heroes.insert(unit.name.clone(), unit.into());
        }
        for module_bindings::House { name, data } in module_bindings::House::iter() {
            pools.houses.insert(name, ron::from_str(&data).unwrap());
        }
        for module_bindings::Ability { name, data } in module_bindings::Ability::iter() {
            pools.abilities.insert(name, ron::from_str(&data).unwrap());
        }
        for module_bindings::Statuses { name, data } in module_bindings::Statuses::iter() {
            pools.statuses.insert(name, ron::from_str(&data).unwrap());
        }
        for module_bindings::Summon { name, data } in module_bindings::Summon::iter() {
            pools.summons.insert(name, ron::from_str(&data).unwrap());
        }
        for module_bindings::Vfx { name, data } in module_bindings::Vfx::iter() {
            pools.vfx.insert(name, ron::from_str(&data).unwrap());
        }

        Self::show_stats(&pools);
        debug!(
            "Cache complete\n{} Heroes\n{} Houses\n{} Abilities\n{} Statuses\n{} Summons\n{} Vfxs",
            pools.heroes.len(),
            pools.houses.len(),
            pools.abilities.len(),
            pools.statuses.len(),
            pools.summons.len(),
            pools.vfx.len()
        );
    }

    fn show_stats(pools: &Pools) {
        let mut abilities: HashMap<String, usize> = default();
        let mut houses: HashMap<String, usize> = default();
        for hero in pools.heroes.values() {
            match &hero.trigger {
                Trigger::Fire {
                    triggers: _,
                    targets: _,
                    effects,
                } => {
                    for effect in effects {
                        for ability in effect.0.find_all_abilities() {
                            match &ability {
                                Effect::UseAbility(ability, _) => {
                                    *abilities.entry(ability.to_owned()).or_default() += 1
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
            for house in hero.houses.split("+") {
                *houses.entry(house.to_owned()).or_default() += 1;
            }
        }
        let houses = houses
            .into_iter()
            .sorted_by_key(|(_, c)| -(*c as i32))
            .map(|(name, count)| {
                let c = pools.house_color(&name).unwrap().as_rgba_u8();
                format!(
                    "{} = {count}",
                    name.custom_color(CustomColor::new(c[0], c[1], c[2]))
                )
            })
            .join("\n");
        debug!("\nHouses:\n----\n{houses}");
        let abilities = abilities
            .into_iter()
            .sorted_by_key(|(_, c)| -(*c as i32))
            .map(|(name, count)| {
                let c = pools.color_by_name(&name).unwrap().as_rgba_u8();
                format!(
                    "{} = {count}",
                    name.custom_color(CustomColor::new(c[0], c[1], c[2]))
                )
            })
            .join("\n");
        debug!("\nAbilities:\n----\n{abilities}");
    }
}

impl Plugin for PoolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), Self::setup);
    }
}
