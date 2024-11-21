use super::*;

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Stats), Self::on_enter);
    }
}

#[derive(Resource)]
struct StatsResource {
    hero_stats: Vec<HeroStat>,
    user_stats: Vec<UserStat>,
}

#[derive(Clone)]
struct HeroStat {
    name: String,
    rarity: Rarity,
    cnt: i32,
    percent: f32,
}
#[derive(Clone)]
struct UserStat {
    name: String,
    id: u64,
    cnt: i32,
    percent: f32,
}

impl StatsPlugin {
    fn on_enter(world: &mut World) {
        let mut units: HashMap<String, i32> = default();
        let mut total_units = 0;
        for battle in cn().db.battle().iter() {
            let team = battle.team_left.get_team();
            for unit in team.units {
                for base in unit.bases {
                    *units.entry(base).or_default() += 1;
                    total_units += 1;
                }
            }
        }
        let mut users: HashMap<u64, i32> = default();
        let mut total_users = 0;
        for battle in cn().db.battle().iter() {
            let player = battle.owner.get_player();
            *users.entry(player.id).or_default() += 1;
            total_users += 1;
        }
        world.insert_resource(StatsResource {
            hero_stats: units
                .into_iter()
                .map(|(name, cnt)| HeroStat {
                    rarity: Rarity::from_base(&name),
                    name,
                    cnt,
                    percent: cnt as f32 / total_units as f32 * 100.0,
                })
                .sorted_by_key(|d| -d.cnt)
                .collect_vec(),
            user_stats: users
                .into_iter()
                .map(|(id, cnt)| UserStat {
                    id,
                    name: cn().db.player().id().find(&id).unwrap().name,
                    cnt,
                    percent: cnt as f32 / total_users as f32 * 100.0,
                })
                .sorted_by_key(|d| -d.cnt)
                .collect_vec(),
        });
    }
    pub fn add_tiles(world: &mut World) {
        Tile::new(Side::Left, |ui, world| {
            Table::new("Hero Stats", |world| {
                world.resource::<StatsResource>().hero_stats.clone()
            })
            .column_base_unit_name("hero", |d: &HeroStat| d.name.clone())
            .column_rarity(|d| d.rarity as i32)
            .column_int("cnt", |d| d.cnt)
            .column_float("percent", |d| d.percent)
            .filter("Common", "rarity", 0.into())
            .filter("Rare", "rarity", 1.into())
            .filter("Epic", "rarity", 2.into())
            .filter("Legendary", "rarity", 3.into())
            .ui(ui, world);
        })
        .pinned()
        .push(world);
        Tile::new(Side::Left, |ui, world| {
            Table::new("Player Stats", |world| {
                world.resource::<StatsResource>().user_stats.clone()
            })
            .column_id("id", |d| d.id)
            .column_cstr("name", |d, _| d.name.cstr_c(VISIBLE_LIGHT))
            .column_int("cnt", |d| d.cnt)
            .column_float("percent", |d| d.percent)
            .ui(ui, world);
        })
        .pinned()
        .push(world);
    }
}
