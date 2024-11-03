use super::*;

pub struct PlayersPlugin;

#[derive(Resource, Default)]
struct PlayersResource {
    players: Vec<TUser>,
    season: u32,
    mode: GameMode,
}

impl PlayersPlugin {
    fn load(world: &mut World) {
        let pr = PlayersResource {
            players: TUser::iter().sorted_by_key(|d| d.id).collect_vec(),
            season: global_settings().season,
            ..default()
        };
        world.insert_resource(pr);
    }
    pub fn add_tiles(world: &mut World) {
        Self::load(world);
        Tile::new(Side::Left, |ui, world| {
            fn get_game_stats(id: u64, mode: u64, season: u32) -> Option<TUserGameStats> {
                let mode: GameMode = mode.into();
                TUserGameStats::filter_by_owner(id)
                    .filter(|d| d.season == season && d.mode.weak_eq(&mode))
                    .next()
            }
            fn get_user_stats(id: u64, season: u32) -> Option<TUserStats> {
                TUserStats::filter_by_owner(id).find(|d| d.season == season)
            }
            world.resource_scope(|world, mut r: Mut<PlayersResource>| {
                season_switcher(&mut r.season, ui);
                game_mode_switcher(&mut r.mode, ui);
                let mode: u64 = r.mode.clone().into();
                let season = r.season;
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
                    .column_cstr_dyn(
                        "played",
                        Box::new(move |u, _| {
                            let secs = Duration::from_micros(
                                get_user_stats(u.id, season)
                                    .map(|u| u.time_played)
                                    .unwrap_or_default(),
                            )
                            .as_secs();
                            format_duration(secs).cstr_cs(VISIBLE_DARK, CstrStyle::Small)
                        }),
                    )
                    .column_int_dyn(
                        "earned",
                        Box::new(move |u| {
                            get_user_stats(u.id, season)
                                .map(|u| u.credits_earned)
                                .unwrap_or_default() as i32
                        }),
                    )
                    .column_int_dyn(
                        "top floor",
                        Box::new(move |u| {
                            get_game_stats(u.id, mode, season)
                                .map(|d| d.runs_max_floor)
                                .unwrap_or_default() as i32
                        }),
                    )
                    .column_float_dyn(
                        "avg floor",
                        Box::new(move |u| {
                            get_game_stats(u.id, mode, season)
                                .map(|d| d.runs_floors as f32 / d.runs_played as f32)
                                .unwrap_or_default()
                        }),
                    )
                    .column_int_dyn(
                        "champion",
                        Box::new(move |u| {
                            get_game_stats(u.id, mode, season)
                                .map(|d| d.champion)
                                .unwrap_or_default() as i32
                        }),
                    )
                    .ui(&r.players, ui, world);
            });
        })
        .transparent()
        .pinned()
        .push(world);
    }
}
