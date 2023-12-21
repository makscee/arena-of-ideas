use crate::module_bindings::User;

use super::*;

pub struct LeaderboardPlugin;

impl Plugin for LeaderboardPlugin {
    fn build(&self, _: &mut App) {
        // app.add_systems(OnEnter(GameState::MainMenu), Self::load);
    }
}

#[derive(Resource, Default, Debug, Clone)]
struct LeaderboardData {
    data: HashMap<LeaderboardType, Vec<(String, usize)>>,
    selected: LeaderboardType,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Default, EnumIter, Display)]
enum LeaderboardType {
    #[default]
    Length,
    Count,
}

impl LeaderboardPlugin {
    pub fn load(world: &mut World) {
        let mut count: HashMap<Identity, usize> = default();
        let mut length: HashMap<Identity, usize> = default();
        for tower in TableTower::iter() {
            *count.entry(tower.owner.clone()).or_default() += 1;
            length
                .entry(tower.owner)
                .and_modify(|v| *v = tower.levels.len().max(*v))
                .or_insert(tower.levels.len());
        }
        let mut lb = LeaderboardData::default();
        let top = lb.data.entry(LeaderboardType::Count).or_default();
        for (i, c) in count.into_iter().sorted_by_key(|(_, v)| *v).rev() {
            top.push((
                User::filter_by_identity(i)
                    .and_then(|u| u.name)
                    .unwrap_or("no_name".to_owned()),
                c,
            ));
        }
        let top = lb.data.entry(LeaderboardType::Length).or_default();
        for (i, c) in length.into_iter().sorted_by_key(|(_, v)| *v).rev() {
            top.push((
                User::filter_by_identity(i)
                    .and_then(|u| u.name)
                    .unwrap_or("no_name".to_owned()),
                c,
            ));
        }
        world.insert_resource(lb)
    }

    pub fn ui(world: &mut World) {
        let ctx = &egui_context(world);
        let mut lb = world.resource::<LeaderboardData>().clone();
        let mut rows = lb.data.remove(&lb.selected).unwrap();
        rows.truncate(10);
        window("LEADERBOARD").show(ctx, |ui| {
            frame(ui, |ui| {
                let lbtypes = LeaderboardType::iter().collect_vec();
                ui.columns(lbtypes.len(), |ui| {
                    for (ind, t) in lbtypes.into_iter().enumerate() {
                        ui[ind].vertical_centered_justified(|ui| {
                            if if t.eq(&lb.selected) {
                                ui.button_primary(t.to_string())
                            } else {
                                ui.button(t.to_string())
                            }
                            .clicked()
                            {
                                world.resource_mut::<LeaderboardData>().selected = t;
                            }
                        });
                    }
                });
                if rows.is_empty() {
                    ui.heading("No data :(");
                } else {
                    for (name, length) in rows {
                        text_dots_text(
                            &name.to_colored(),
                            &length.to_string().add_color(white()),
                            ui,
                        );
                    }
                }
            });
        });
    }
}
