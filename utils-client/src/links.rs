use std::collections::VecDeque;

use super::*;

static PARENT_CHILD_MAP: OnceCell<Mutex<HashMap<u64, HashSet<u64>>>> = OnceCell::new();
static CHILD_PARENT_MAP: OnceCell<Mutex<HashMap<u64, HashSet<u64>>>> = OnceCell::new();

pub fn parent_child_map() -> MutexGuard<'static, HashMap<u64, HashSet<u64>>> {
    PARENT_CHILD_MAP.get().unwrap().lock()
}
pub fn child_parent_map() -> MutexGuard<'static, HashMap<u64, HashSet<u64>>> {
    CHILD_PARENT_MAP.get().unwrap().lock()
}
pub fn get_children(id: u64) -> HashSet<u64> {
    parent_child_map().get(&id).cloned().unwrap_or_default()
}
pub fn get_parents(id: u64) -> HashSet<u64> {
    child_parent_map().get(&id).cloned().unwrap_or_default()
}
pub fn get_children_recursive(id: u64) -> HashSet<u64> {
    let mut q = VecDeque::from([id]);
    let mut result: HashSet<u64> = default();
    while let Some(id) = q.pop_front() {
        for id in get_children(id) {
            if !result.insert(id) {
                continue;
            }
            q.push_back(id);
        }
    }
    result
}
pub fn get_parents_recursive(id: u64) -> HashSet<u64> {
    let mut q = VecDeque::from([id]);
    let mut result: HashSet<u64> = default();
    while let Some(id) = q.pop_front() {
        for id in get_parents(id) {
            if !result.insert(id) {
                continue;
            }
            q.push_back(id);
        }
    }
    result
}

pub fn links_init() {
    PARENT_CHILD_MAP.set(Mutex::new(default())).unwrap();
    CHILD_PARENT_MAP.set(Mutex::new(default())).unwrap();
}
pub fn links_add(child: u64, parent: u64) {
    parent_child_map().entry(parent).or_default().insert(child);
    child_parent_map().entry(child).or_default().insert(parent);
}
pub fn links_remove(child: u64, parent: u64) {
    if let Some(children) = parent_child_map().get_mut(&parent) {
        children.remove(&child);
    }
    if let Some(parents) = child_parent_map().get_mut(&child) {
        parents.remove(&parent);
    }
}
pub fn links_delete_id(id: u64) {
    if let Some(children) = parent_child_map().remove(&id) {
        let mut m = child_parent_map();
        for child in children {
            if let Some(parents) = m.get_mut(&child) {
                parents.remove(&id);
            }
        }
    }
}

pub trait IdLinkExt {
    fn add_parent(self, parent: u64);
    fn add_child(self, child: u64);
    fn is_parent_of(self, child: u64) -> bool;
    fn is_child_of(self, parent: u64) -> bool;
    fn any_parent(self) -> Option<u64>;
}

impl IdLinkExt for u64 {
    fn add_parent(self, parent: u64) {
        links_add(self, parent);
    }
    fn add_child(self, child: u64) {
        links_add(child, self);
    }
    fn is_parent_of(self, child: u64) -> bool {
        get_children(self).contains(&child)
    }
    fn is_child_of(self, parent: u64) -> bool {
        get_parents(self).contains(&parent)
    }
    fn any_parent(self) -> Option<u64> {
        get_parents(self).iter().copied().next()
    }
}
