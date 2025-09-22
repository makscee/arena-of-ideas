use super::*;

pub struct GameTimerPlugin;
impl Plugin for GameTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(time: Res<Time>) {
    gt().update(time.delta_secs(), time.elapsed_secs_f64());
}
