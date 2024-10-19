use super::*;

pub struct ServerSyncPlugin;

impl Plugin for ServerSyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::ServerSync), Self::sync);
    }
}

impl ServerSyncPlugin {
    fn sync() {
        info!("Sync assets start");
        let ga = game_assets().clone();
        let gs = ga.global_settings;

        let mut representations: HashMap<String, TRepresentation> = default();
        let mut units: Vec<TBaseUnit> = default();
        let mut packed_units = ga.heroes.into_values().collect_vec();
        packed_units.push(ga.ghost);
        for house in ga.houses.values() {
            for mut summon in house.summons.iter().cloned() {
                summon.rarity = -1;
                packed_units.push(summon);
            }
        }
        for unit in packed_units {
            if representations
                .insert(
                    unit.name.clone(),
                    TRepresentation {
                        id: unit.name.clone(),
                        data: ron::to_string(&unit.representation).unwrap(),
                    },
                )
                .is_some()
            {
                panic!("Duplicate representation {:?}", unit);
            }
            units.push(unit.into());
        }

        let houses = ga.houses.into_values().map(|h| h.into()).collect_vec();
        let abilities = ga.abilities.into_values().map(|a| a.into()).collect_vec();
        let statuses = ga.statuses.into_values().map(|s| s.into()).collect_vec();
        let representations = representations.into_values().collect_vec();
        upload_assets(gs, representations, units, houses, abilities, statuses);
        once_on_upload_assets(|_, _, status, _, _, _, _, _, _| {
            match status {
                spacetimedb_sdk::reducer::Status::Committed => {
                    info!("Sync successful")
                }
                spacetimedb_sdk::reducer::Status::Failed(e) => error!("Sync failed: {e}"),
                _ => panic!(),
            };
            OperationsPlugin::add(|world| app_exit(world));
        });
    }
}
