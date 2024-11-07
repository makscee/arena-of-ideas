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

        let mut representations: HashMap<String, Representation> = default();
        let mut units: Vec<TBaseUnit> = default();
        representations.insert(ga.ghost.name.clone(), ga.ghost.representation.clone());
        units.push({
            let mut unit: TBaseUnit = ga.ghost.into();
            unit.pool = UnitPool::Summon;
            unit
        });
        for house in ga.houses.values() {
            for summon in house.summons.iter().cloned() {
                representations.insert(summon.name.clone(), summon.representation.clone());
                let mut unit: TBaseUnit = summon.into();
                unit.pool = UnitPool::Summon;
                units.push(unit);
            }
        }
        for hero in ga.heroes.into_values() {
            representations.insert(hero.name.clone(), hero.representation.clone());
            units.push(hero.into());
        }

        let houses = ga.houses.into_values().map(|h| h.into()).collect_vec();
        let abilities = ga.abilities.into_values().map(|a| a.into()).collect_vec();
        let statuses = ga.statuses.into_values().map(|s| s.into()).collect_vec();
        cn().reducers
            .upload_assets(gs, units, houses, abilities, statuses)
            .unwrap();
        cn().reducers.on_upload_assets(|e, _, _, _, _, _| {
            e.event.on_success(|_| info!("Sync successful"));
            OperationsPlugin::add(|world| app_exit(world));
        });
    }
}
