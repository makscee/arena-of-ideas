use super::*;

#[derive(Resource, Default)]
pub struct IdEntityLinks {
    id_entity: HashMap<u64, Entity>,
    entity_id: HashMap<Entity, u64>,
}

pub trait WorldNodeExt {
    fn add_id_link(&mut self, id: u64, entity: Entity);
    fn get_id_link(&self, id: u64) -> Option<Entity>;
    fn get_entity_link(&self, entity: Entity) -> Option<u64>;
    fn clear_id_link(&mut self, id: u64);
}

impl WorldNodeExt for World {
    fn add_id_link(&mut self, id: u64, entity: Entity) {
        let mut links = self.get_resource_or_insert_with::<IdEntityLinks>(|| default());
        links.id_entity.insert(id, entity);
        links.entity_id.insert(entity, id);
    }
    fn get_id_link(&self, id: u64) -> Option<Entity> {
        self.get_resource::<IdEntityLinks>()
            .and_then(|r| r.id_entity.get(&id))
            .copied()
    }
    fn get_entity_link(&self, entity: Entity) -> Option<u64> {
        self.get_resource::<IdEntityLinks>()
            .and_then(|r| r.entity_id.get(&entity))
            .copied()
    }
    fn clear_id_link(&mut self, id: u64) {
        if let Some(mut r) = self.get_resource_mut::<IdEntityLinks>() {
            if let Some(entity) = r.id_entity.remove(&id) {
                r.entity_id.remove(&entity);
            }
        }
    }
}
