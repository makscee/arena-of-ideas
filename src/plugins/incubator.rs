use super::*;

pub struct IncubatorPlugin;

impl IncubatorPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            let data = TIncubator::iter().collect_vec();
            Table::new("Incubator")
                .title()
                .column_base_unit("unit", |d: &TIncubator| d.unit.clone())
                .column_user_click("owner", |d| d.owner)
                .ui(&data, ui, world);
        })
        .pinned()
        .push(world);
    }
}
