use super::*;

#[derive(Default)]
pub struct GameOverSystem {
    pub victory: bool,
    pub need_restart: bool,
}

impl GameOverSystem {
    pub fn new() -> Self {
        default()
    }

    pub fn init(world: &mut legion::World, resources: &mut Resources) {
        let mut node = Node::default();
        UnitSystem::draw_all_units_to_node(&hashset! {Faction::Team}, &mut node, world, resources);
        resources.tape_player.tape.persistent_node = node;
    }
}

impl System for GameOverSystem {}
