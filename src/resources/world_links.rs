use super::*;

pub trait WorldLinks {
    fn init_links(&mut self);
    fn parents_children_map(&self) -> &HashMap<u64, HashSet<u64>>;
    fn children_parents_map(&self) -> &HashMap<u64, HashSet<u64>>;
    fn id_to_entity_map(&self) -> &HashMap<u64, Entity>;
    fn entity_to_id_map(&self) -> &HashMap<Entity, u64>;
    fn link_parent_child(&mut self, parent: u64, child: u64);
    fn set_link_rating(&mut self, parent: u64, child: u64, rating: i32);
    fn get_link_rating(&self, parent: u64, child: u64) -> Option<i32>;
    fn unlink_parent_child(&mut self, parent: u64, child: u64) -> bool;
    fn id_entity(&self, id: u64) -> Option<Entity>;
    fn entity_id(&self, entity: Entity) -> Option<u64>;
    fn link_id_entity(&mut self, id: u64, entity: Entity);
    fn despawn_entity(&mut self, entity: Entity);
}
#[derive(Default, Resource)]
struct WorldLinksResource {
    parent_to_child: HashMap<u64, HashSet<u64>>,
    child_to_parent: HashMap<u64, HashSet<u64>>,
    links_rating: HashMap<(u64, u64), i32>,
    id_to_entity: HashMap<u64, Entity>,
    entity_to_id: HashMap<Entity, u64>,
}
impl WorldLinks for World {
    fn init_links(&mut self) {
        self.init_resource::<WorldLinksResource>();
    }
    fn parents_children_map(&self) -> &HashMap<u64, HashSet<u64>> {
        &self.resource::<WorldLinksResource>().parent_to_child
    }
    fn children_parents_map(&self) -> &HashMap<u64, HashSet<u64>> {
        &self.resource::<WorldLinksResource>().child_to_parent
    }
    fn id_to_entity_map(&self) -> &HashMap<u64, Entity> {
        &self.resource::<WorldLinksResource>().id_to_entity
    }
    fn entity_to_id_map(&self) -> &HashMap<Entity, u64> {
        &self.resource::<WorldLinksResource>().entity_to_id
    }
    fn link_parent_child(&mut self, parent: u64, child: u64) {
        let mut r = self.resource_mut::<WorldLinksResource>();
        r.parent_to_child.entry(parent).or_default().insert(child);
        r.child_to_parent.entry(child).or_default().insert(parent);
    }
    fn set_link_rating(&mut self, parent: u64, child: u64, rating: i32) {
        let mut r = self.resource_mut::<WorldLinksResource>();
        r.links_rating.insert((parent, child), rating);
    }
    fn get_link_rating(&self, parent: u64, child: u64) -> Option<i32> {
        self.resource::<WorldLinksResource>()
            .links_rating
            .get(&(parent, child))
            .copied()
    }
    fn unlink_parent_child(&mut self, parent: u64, child: u64) -> bool {
        let mut r = self.resource_mut::<WorldLinksResource>();
        let mut removed = false;
        if let Some(children) = r.parent_to_child.get_mut(&parent) {
            removed = true;
            children.remove(&child);
        }
        if let Some(parents) = r.child_to_parent.get_mut(&child) {
            removed = true;
            parents.remove(&parent);
        }
        removed
    }
    fn despawn_entity(&mut self, entity: Entity) {
        let mut r = self.resource_mut::<WorldLinksResource>();
        if let Some(id) = r.entity_to_id.get(&entity).copied() {
            r.id_to_entity.remove(&id);
            if let Some(children) = r.parent_to_child.remove(&id) {
                for child in children {
                    if let Some(parents) = r.child_to_parent.get_mut(&child) {
                        parents.remove(&id);
                    }
                }
            }
        }
        r.entity_to_id.remove(&entity);
    }
    fn id_entity(&self, id: u64) -> Option<Entity> {
        self.resource::<WorldLinksResource>()
            .id_to_entity
            .get(&id)
            .copied()
    }
    fn entity_id(&self, entity: Entity) -> Option<u64> {
        self.resource::<WorldLinksResource>()
            .entity_to_id
            .get(&entity)
            .copied()
    }
    fn link_id_entity(&mut self, id: u64, entity: Entity) {
        let mut r = self.resource_mut::<WorldLinksResource>();
        r.entity_to_id.insert(entity, id);
        r.id_to_entity.insert(id, entity);
    }
}

pub trait IdLinkExt {
    fn add_parent(self, world: &mut World, parent: u64);
    fn add_child(self, world: &mut World, child: u64);
    fn is_parent_of(self, world: &World, child: u64) -> bool;
    fn is_child_of(self, world: &World, parent: u64) -> bool;
}

impl IdLinkExt for u64 {
    fn add_parent(self, world: &mut World, parent: u64) {
        world.link_parent_child(parent, self);
    }
    fn add_child(self, world: &mut World, child: u64) {
        world.link_parent_child(self, child);
    }
    fn is_parent_of(self, world: &World, child: u64) -> bool {
        if let Some(children) = world.parents_children_map().get(&self) {
            children.contains(&child)
        } else {
            false
        }
    }
    fn is_child_of(self, world: &World, parent: u64) -> bool {
        if let Some(children) = world.children_parents_map().get(&self) {
            children.contains(&parent)
        } else {
            false
        }
    }
}
