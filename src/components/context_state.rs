use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextState {
    pub name: String,
    pub statuses: HashMap<String, i32>,
    pub status_change_t: HashMap<String, usize>,
    pub t: usize,
    pub ability_vars: HashMap<AbilityName, Vars>,
    pub vars: Vars,
    #[serde(skip)]
    pub parent: Option<legion::Entity>,
}

impl ContextState {
    pub fn new(name: String, parent: Option<legion::Entity>) -> Self {
        Self {
            name,
            statuses: default(),
            ability_vars: default(),
            vars: default(),
            parent,
            status_change_t: default(),
            t: default(),
        }
    }

    pub fn try_get<'a>(
        entity: legion::Entity,
        world: &'a legion::World,
    ) -> Option<&'a ContextState> {
        if let Ok(entry) = world.entry_ref(entity) {
            if let Ok(state) = entry.into_component::<ContextState>() {
                return Some(state);
            }
        }
        None
    }

    pub fn get<'a>(entity: legion::Entity, world: &'a legion::World) -> &'a ContextState {
        if let Ok(entry) = world.entry_ref(entity) {
            if let Ok(state) = entry.into_component::<ContextState>() {
                return state;
            }
        }
        panic!("No state for {entity:?}")
    }

    pub fn get_mut<'a>(
        entity: legion::Entity,
        world: &'a mut legion::World,
    ) -> &'a mut ContextState {
        if let Ok(entry) = world.entry_mut(entity) {
            if let Ok(state) = entry.into_component_mut::<ContextState>() {
                return state;
            }
        }
        panic!("No state")
    }

    pub fn get_var<'a>(&'a self, var: &VarName, world: &'a legion::World) -> Option<&'a Var> {
        self.vars.try_get(var).or(self
            .parent
            .and_then(|x| Self::get(x, world).get_var(var, world)))
    }

    pub fn get_int(&self, var: &VarName, world: &legion::World) -> i32 {
        let value = self.get_var(var, world).unwrap();
        match value {
            Var::Int(value) => *value,
            _ => panic!("Wrong var type {value:?}"),
        }
    }

    pub fn try_get_int(&self, var: &VarName, world: &legion::World) -> Option<i32> {
        self.get_var(var, world).map(|x| match x {
            Var::Int(value) => *value,
            _ => panic!("Wrong var type {x:?}"),
        })
    }

    pub fn get_faction(&self, var: &VarName, world: &legion::World) -> Faction {
        let value = self.get_var(var, world).unwrap();
        match value {
            Var::Faction(value) => *value,
            _ => panic!("Wrong var type {value:?}"),
        }
    }
}
