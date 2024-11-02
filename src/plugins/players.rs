use super::*;

pub struct PlayersPlugin;

#[derive(Resource, Default)]
struct PlayersResource {
    players: Vec<TUser>,
}
fn rm(world: &mut World) -> Mut<PlayersResource> {
    world.resource_mut::<PlayersResource>()
}

impl PlayersPlugin {
    fn load(world: &mut World) {
        let pr = PlayersResource {
            players: TUser::iter().sorted_by_key(|d| d.id).collect_vec(),
        };
        world.insert_resource(pr);
    }
    pub fn add_tiles(world: &mut World) {
        Self::load(world);
        Tile::new(Side::Left, |ui, world| {
            world.resource_scope(|world, r: Mut<PlayersResource>| {
                Table::new("Players")
                    .column_user_click("name", |d: &TUser| d.id)
                    .column_cstr("online", |d, _| {
                        if d.online {
                            "online".cstr_c(VISIBLE_LIGHT)
                        } else {
                            if d.last_login == 0 {
                                "-".cstr_c(VISIBLE_DARK)
                            } else {
                                format_timestamp(d.last_login)
                                    .cstr_cs(VISIBLE_DARK, CstrStyle::Small)
                            }
                        }
                    })
                    .ui(&r.players, ui, world);
            });
        })
        .transparent()
        .pinned()
        .push(world);
    }
}
