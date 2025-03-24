use super::*;

pub struct TeamEditorPlugin;

#[derive(Resource)]
struct TeamEditorData {
    entity: Entity,
}

fn rm(world: &mut World) -> Result<Mut<TeamEditorData>, ExpressionError> {
    world
        .get_resource_mut::<TeamEditorData>()
        .to_e("Team not loaded")
}
impl TeamEditorPlugin {
    pub fn add_panes() {
        TilePlugin::op(|tree| {
            if let Some(id) = tree.tiles.find_pane(&Pane::TeamEditorRoster) {
                tree.tiles.remove(id);
            }
            let roster = tree.tiles.insert_pane(Pane::TeamEditorRoster);
            let id = tree.tiles.insert_horizontal_tile([roster].into());
            tree.add_to_root(id).log();
        });
    }
    pub fn load_team(team: Team, world: &mut World) {
        let entity = world.spawn_empty().id();
        team.unpack(entity, world);
        world.insert_resource(TeamEditorData { entity });
    }
    pub fn load_team_entity(entity: Entity, world: &mut World) {
        if world.get::<Team>(entity).is_none() {
            format!("No team component on {entity}").notify_error(world);
            return;
        }
        world.insert_resource(TeamEditorData { entity });
    }
    pub fn add_unit(entity: Entity, world: &mut World) -> Result<(), ExpressionError> {
        let team = rm(world)?.entity;
        let unit = world.get::<Unit>(entity).to_e("Unit component not found")?;
        let context = Context::new_world(world);
        let house = context
            .find_parent_component::<House>(entity)
            .to_e("House parent not found")?;
        if let Some(team_house) = context
            .children_components::<House>(team)
            .into_iter()
            .find(|h| h.name == house.name)
        {
            let house = team_house.entity();
            let mut unit = Unit::pack(entity, world).to_e("Failed to pack Unit")?;
            unit.clear_ids();
            unit.unpack(world.spawn_empty().set_parent(house).id(), world);
        } else {
            let mut house = House::pack(house.entity(), world).to_e("Failed to pack House")?;
            house.units.retain(|u| u.name == unit.name);
            house.clear_ids();
            house.unpack(world.spawn_empty().set_parent(team).id(), world);
        }
        Ok(())
    }
    pub fn pane_roster(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let Some(team) = world.get_resource::<TeamEditorData>().map(|d| d.entity) else {
            "No team loaded".cstr_c(RED).label(ui);
            return Ok(());
        };
        Team::get(team, world)
            .unwrap()
            .show(None, &Context::new_world(world), ui);
        let context = Context::new_world(world);
        for house in context.children_components::<House>(team) {
            let color = house.color_load(world)?.color.c32();
            ui.collapsing(
                house
                    .name
                    .cstr_cs(color, CstrStyle::Bold)
                    .widget(1.0, ui.style()),
                |ui| {
                    for unit in context.children_components::<Unit>(house.entity()) {
                        unit.name.cstr_c(color).label(ui);
                    }
                },
            );
        }
        Ok(())
    }
}
