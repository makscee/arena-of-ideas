use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, _: &mut App) {}
}

#[derive(Resource)]
struct MatchData {
    g: i32,
    shop_units: Vec<Option<Unit>>,
    world: World,
}

impl MatchPlugin {
    pub fn load_match_data(id: u64, world: &mut World) {
        let m = Match::from_table(NodeDomain::Match, id).unwrap();
        let mut match_world = World::new();
        m.team
            .unwrap()
            .unpack(match_world.spawn_empty().id(), &mut match_world.commands());
        match_world.flush();
        let shop_units = m
            .shop_case
            .into_iter()
            .map(|d| {
                if !d.sold {
                    let id = d.unit_id;
                    let unit = Unit::from_table(NodeDomain::Core, id);
                    if unit.is_none() {
                        error!("Core unit#{id} not found");
                    }
                    unit
                } else {
                    None
                }
            })
            .collect_vec();
        world.insert_resource(MatchData {
            g: 13,
            shop_units,
            world: match_world,
        });
    }
    pub fn open_shop_window(world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        Window::new("Match", move |ui, world| {
            let mut md = world.remove_resource::<MatchData>().unwrap();

            for (entity, slot) in md.world.query::<(Entity, &UnitSlot)>().iter(&md.world) {
                let slot = slot.slot as usize;
            }

            world.insert_resource(md);
        })
        .default_width(500.0)
        .push(world);
    }
}
