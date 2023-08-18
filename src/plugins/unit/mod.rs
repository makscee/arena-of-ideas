use super::*;

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, Self::update_units);
    }
}

impl UnitPlugin {
    fn update_units(time: Res<Time>, mut query: Query<&mut Transform, With<Unit>>) {
        for mut t in query.iter_mut() {
            t.translation.y = time.elapsed_seconds().sin() * 2.0;
        }
    }
}

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
pub struct PackedUnit {
    pub representation: Representation,
    pub state: VarState,
}

impl PackedUnit {
    pub fn unpack(self, world: &mut World) {
        let entity = self.representation.unpack(None, world);
        world.entity_mut(entity).insert(Unit).insert(self.state);
    }
}

#[derive(Resource, Debug)]
pub struct UnitHandle(pub Handle<PackedUnit>);

#[derive(Component)]
pub struct Unit;
