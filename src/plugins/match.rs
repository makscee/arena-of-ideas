use super::*;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, _: &mut App) {}
}

#[derive(Resource)]
struct MatchData {
    g: i32,
    shop_case: Vec<ShopCaseUnit>,
    team_world: World,
    core_world: World,
}

impl MatchPlugin {
    pub fn load_match_data(id: u64, world: &mut World) {
        let m = Match::from_table(NodeDomain::Match, id).unwrap();
        let mut team_world = World::new();
        m.team
            .unwrap()
            .unpack(team_world.spawn_empty().id(), &mut team_world);

        let mut core_world = World::new();
        for house in NodeDomain::Core.filter_by_kind(NodeKind::House) {
            let house = House::from_table(NodeDomain::Core, house.id).unwrap();
            house.unpack(core_world.spawn_empty().id(), &mut core_world);
        }

        let shop_case = m.shop_case;
        world.insert_resource(MatchData {
            g: 13,
            shop_case,
            team_world,
            core_world,
        });
    }
    pub fn open_shop_window(world: &mut World) {
        if !world.contains_resource::<MatchData>() {
            error!("Match not loaded");
            return;
        }
        Window::new("Match", move |ui, world| {
            let mut md = world.remove_resource::<MatchData>().unwrap();
            let shop_slots = md.shop_case.len();
            let rect = ui.available_rect_before_wrap();
            for (i, sc) in md.shop_case.iter().enumerate() {
                let rect = show_slot(i, shop_slots, false, ui).rect;
                if !sc.sold {
                    let entity = md.core_world.get_id_link(sc.unit_id).unwrap();
                    if let Some(rep) = md.core_world.get::<Representation>(entity) {
                        let context = &Context::new_world(&md.core_world).set_owner(entity).take();
                        RepresentationPlugin::paint_rect(rect, context, &rep.material, ui).log();
                    }
                }
            }
            for (entity, slot) in md
                .team_world
                .query::<(Entity, &UnitSlot)>()
                .iter(&md.team_world)
            {
                let slot = slot.slot as usize;
            }

            world.insert_resource(md);
        })
        .default_width(800.0)
        .default_height(600.0)
        .push(world);
    }
}
