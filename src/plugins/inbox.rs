use super::*;

pub struct InboxPlugin {}

impl InboxPlugin {
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            Notification::show_all_table(ui, world)
        })
        .transparent()
        .pinned()
        .push(world)
    }
}
