use super::*;

pub trait WorldLinks {
    fn init_links(&mut self);
    fn parents_children_map(&self) -> &HashMap<u64, HashSet<u64>>;
    fn children_parents_map(&self) -> &HashMap<u64, HashSet<u64>>;
    fn parents_children_all_map(&self) -> &HashMap<u64, HashSet<u64>>;
    fn children_parents_all_map(&self) -> &HashMap<u64, HashSet<u64>>;
    fn id_to_entity_map(&self) -> &HashMap<u64, Entity>;
    fn entity_to_id_map(&self) -> &HashMap<Entity, u64>;
    fn links_rating_all_map(&self) -> &HashMap<(u64, u64), (i32, bool)>;
    fn id_kind_map(&self) -> &HashMap<u64, NodeKind>;
    fn link_parent_child(&mut self, parent: u64, child: u64);
    fn set_link_rating(&mut self, parent: u64, child: u64, rating: i32, solid: bool);
    fn get_link_rating(&self, parent: u64, child: u64) -> Option<i32>;
    fn get_any_link_rating(&self, parent: u64, child: u64) -> Option<(i32, bool)>;
    fn unlink_parent_child(&mut self, parent: u64, child: u64) -> bool;
    fn id_entity(&self, id: u64) -> Option<Entity>;
    fn entity_id(&self, entity: Entity) -> Option<u64>;
    fn link_id_entity(&mut self, id: u64, entity: Entity);
    fn set_id_kind(&mut self, id: u64, kind: NodeKind);
    fn despawn_entity(&mut self, entity: Entity);
}
#[derive(Default, Resource)]
struct WorldLinksResource {
    parent_to_child: HashMap<u64, HashSet<u64>>,
    child_to_parent: HashMap<u64, HashSet<u64>>,
    parent_to_child_all: HashMap<u64, HashSet<u64>>,
    child_to_parent_all: HashMap<u64, HashSet<u64>>,
    links_rating: HashMap<(u64, u64), i32>,
    links_rating_all: HashMap<(u64, u64), (i32, bool)>,
    id_to_entity: HashMap<u64, Entity>,
    entity_to_id: HashMap<Entity, u64>,
    id_kind: HashMap<u64, NodeKind>,
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
    fn parents_children_all_map(&self) -> &HashMap<u64, HashSet<u64>> {
        &self.resource::<WorldLinksResource>().parent_to_child_all
    }
    fn children_parents_all_map(&self) -> &HashMap<u64, HashSet<u64>> {
        &self.resource::<WorldLinksResource>().child_to_parent_all
    }
    fn id_to_entity_map(&self) -> &HashMap<u64, Entity> {
        &self.resource::<WorldLinksResource>().id_to_entity
    }
    fn entity_to_id_map(&self) -> &HashMap<Entity, u64> {
        &self.resource::<WorldLinksResource>().entity_to_id
    }
    fn links_rating_all_map(&self) -> &HashMap<(u64, u64), (i32, bool)> {
        &self.resource::<WorldLinksResource>().links_rating_all
    }
    fn id_kind_map(&self) -> &HashMap<u64, NodeKind> {
        &self.resource::<WorldLinksResource>().id_kind
    }
    fn link_parent_child(&mut self, parent: u64, child: u64) {
        let mut r = self.resource_mut::<WorldLinksResource>();
        r.parent_to_child.entry(parent).or_default().insert(child);
        r.child_to_parent.entry(child).or_default().insert(parent);
        r.parent_to_child_all
            .entry(parent)
            .or_default()
            .insert(child);
        r.child_to_parent_all
            .entry(child)
            .or_default()
            .insert(parent);
    }
    fn set_link_rating(&mut self, parent: u64, child: u64, rating: i32, solid: bool) {
        let mut r = self.resource_mut::<WorldLinksResource>();
        if solid {
            r.links_rating.insert((parent, child), rating);
            r.parent_to_child.entry(parent).or_default().insert(child);
            r.child_to_parent.entry(child).or_default().insert(parent);
        } else {
            r.links_rating.remove(&(parent, child));
            if let Some(children) = r.parent_to_child.get_mut(&parent) {
                children.remove(&child);
            }
            if let Some(parents) = r.child_to_parent.get_mut(&child) {
                parents.remove(&parent);
            }
        }
        r.links_rating_all.insert((parent, child), (rating, solid));
        r.parent_to_child_all
            .entry(parent)
            .or_default()
            .insert(child);
        r.child_to_parent_all
            .entry(child)
            .or_default()
            .insert(parent);
    }
    fn get_link_rating(&self, parent: u64, child: u64) -> Option<i32> {
        self.resource::<WorldLinksResource>()
            .links_rating
            .get(&(parent, child))
            .copied()
    }
    fn get_any_link_rating(&self, parent: u64, child: u64) -> Option<(i32, bool)> {
        self.resource::<WorldLinksResource>()
            .links_rating_all
            .get(&(parent, child))
            .copied()
    }
    fn unlink_parent_child(&mut self, parent: u64, child: u64) -> bool {
        let mut r = self.resource_mut::<WorldLinksResource>();
        let mut removed = false;
        if let Some(children) = r.parent_to_child.get_mut(&parent) {
            removed = children.remove(&child) || removed;
        }
        if let Some(parents) = r.child_to_parent.get_mut(&child) {
            removed = parents.remove(&parent) || removed;
        }
        if let Some(children) = r.parent_to_child_all.get_mut(&parent) {
            removed = children.remove(&child) || removed;
        }
        if let Some(parents) = r.child_to_parent_all.get_mut(&child) {
            removed = parents.remove(&parent) || removed;
        }
        r.links_rating.remove(&(parent, child));
        r.links_rating_all.remove(&(parent, child));
        removed
    }
    fn despawn_entity(&mut self, entity: Entity) {
        self.entity_mut(entity).despawn();
        let mut r = self.resource_mut::<WorldLinksResource>();
        if let Some(id) = r.entity_to_id.get(&entity).copied() {
            r.id_to_entity.remove(&id);
            r.id_kind.remove(&id);

            if let Some(children) = r.parent_to_child.remove(&id) {
                for child in children {
                    if let Some(parents) = r.child_to_parent.get_mut(&child) {
                        parents.remove(&id);
                    }
                    r.links_rating.remove(&(id, child));
                    r.links_rating_all.remove(&(id, child));
                }
            }

            if let Some(children) = r.parent_to_child_all.remove(&id) {
                for child in children {
                    if let Some(parents) = r.child_to_parent_all.get_mut(&child) {
                        parents.remove(&id);
                    }
                }
            }

            if let Some(parents) = r.child_to_parent.remove(&id) {
                for parent in parents {
                    if let Some(children) = r.parent_to_child.get_mut(&parent) {
                        children.remove(&id);
                    }
                    r.links_rating.remove(&(parent, id));
                    r.links_rating_all.remove(&(parent, id));
                }
            }

            if let Some(parents) = r.child_to_parent_all.remove(&id) {
                for parent in parents {
                    if let Some(children) = r.parent_to_child_all.get_mut(&parent) {
                        children.remove(&id);
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
    fn set_id_kind(&mut self, id: u64, kind: NodeKind) {
        let mut r = self.resource_mut::<WorldLinksResource>();
        r.id_kind.insert(id, kind);
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
