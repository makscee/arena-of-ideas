use std::sync::LazyLock;

use bevy::utils::hashbrown::HashMap;

use super::*;

pub type NID = u64;

static ENTITY_NID_MAP: LazyLock<Mutex<HashMap<Entity, NID>>> = LazyLock::new(|| default());
static NID_ENTITY_MAP: LazyLock<Mutex<HashMap<NID, Entity>>> = LazyLock::new(|| default());

pub fn entity_nid_link(entity: Entity, nid: NID) {
    ENTITY_NID_MAP.lock().insert(entity, nid);
    NID_ENTITY_MAP.lock().insert(nid, entity);
}
pub fn entity_nid(entity: Entity) -> Option<NID> {
    ENTITY_NID_MAP.lock().get(&entity).copied()
}
pub fn nid_entity(nid: NID) -> Option<Entity> {
    NID_ENTITY_MAP.lock().get(&nid).copied()
}

pub trait NidEntityExt {
    fn entity(self) -> Entity;
}
impl NidEntityExt for NID {
    fn entity(self) -> Entity {
        nid_entity(self).unwrap()
    }
}

pub trait EntityNidExt {
    fn nid(self) -> NID;
}
impl EntityNidExt for Entity {
    fn nid(self) -> NID {
        entity_nid(self).unwrap()
    }
}
