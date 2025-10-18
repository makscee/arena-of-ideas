use chrono::Utc;
use spacetimedb_lib::Timestamp;

use super::*;

pub fn delta_time(world: &World) -> f32 {
    world.resource::<Time>().delta_secs()
}
pub fn elapsed_seconds(world: &World) -> f32 {
    world.resource::<Time>().elapsed_secs()
}
pub fn global_settings() -> &'static GlobalSettings {
    if is_connected() {
        global_settings_local()
    } else {
        global_settings_local()
    }
}
pub fn app_exit(world: &mut World) {
    world
        .get_resource_mut::<bevy::prelude::Messages<bevy::app::AppExit>>()
        .unwrap()
        .send(bevy::app::AppExit::Success);
}
pub fn app_exit_op() {
    OperationsPlugin::add(app_exit)
}
pub fn cur_state(world: &World) -> GameState {
    *world.resource::<State<GameState>>().get()
}
pub fn rng_seeded(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}
pub fn show_daily_refresh_timer(ui: &mut Ui) {
    let now = Utc::now().timestamp();
    let til_refresh = (now / 86400 + 1) * 86400 - now;
    format!(
        "Refresh in {}",
        format_duration(til_refresh as u64).cstr_cs(high_contrast_text(), CstrStyle::Bold)
    )
    .label(ui);
}
static NEXT_ID: Mutex<u64> = Mutex::new(0);
pub fn next_id() -> u64 {
    let ts = Timestamp::now().to_micros_since_unix_epoch() as u64;
    let mut next_id = NEXT_ID.lock();
    if *next_id >= ts {
        *next_id += 1;
        *next_id
    } else {
        *next_id = ts;
        ts
    }
}
pub fn set_next_id(id: u64) {
    *NEXT_ID.lock() = id;
}
// pub trait CstrExt {
//     fn print(&self);
//     fn info(&self);
//     fn debug(&self);
// }

// impl CstrExt for Cstr {
//     fn print(&self) {
//         println!("{}", self.to_colored())
//     }
//     fn info(&self) {
//         info!("{}", self.to_colored())
//     }
//     fn debug(&self) {
//         debug!("{}", self.to_colored())
//     }
// }
