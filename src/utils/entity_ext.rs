use super::*;
pub trait EntityExt {
    fn id(self, world: &World) -> u64;
    fn get_children(self, world: &World) -> Vec<Entity>;
    fn get_children_recursive(self, world: &World) -> Vec<Entity>;
    fn get_parents(self, world: &World) -> Vec<Entity>;
    fn to_value(self) -> VarValue;
}

impl EntityExt for Entity {
    fn id(self, world: &World) -> u64 {
        world.get_entity_link(self).unwrap()
    }
    fn get_children(self, world: &World) -> Vec<Entity> {
        let Some(id) = world.get_entity_link(self) else {
            error!("{self} not linked to world");
            return default();
        };
        get_children(id)
            .into_iter()
            .filter_map(|id| id.entity(world))
            .collect()
    }
    fn get_children_recursive(self, world: &World) -> Vec<Entity> {
        let Some(id) = world.get_entity_link(self) else {
            error!("{self} not linked to world");
            return default();
        };
        get_children_recursive(id)
            .into_iter()
            .filter_map(|id| id.entity(world))
            .collect()
    }
    fn get_parents(self, world: &World) -> Vec<Entity> {
        let Some(id) = world.get_entity_link(self) else {
            error!("{self} not linked to world");
            return default();
        };
        get_parents(id)
            .into_iter()
            .filter_map(|id| id.entity(world))
            .collect()
    }
    fn to_value(self) -> VarValue {
        VarValue::Entity(self.to_bits())
    }
}

pub trait EntityVecVarValue {
    fn vec_to_value(self) -> VarValue;
}

impl EntityVecVarValue for Vec<Entity> {
    fn vec_to_value(self) -> VarValue {
        self.into_iter().map(|e| e.to_value()).collect_vec().into()
    }
}
