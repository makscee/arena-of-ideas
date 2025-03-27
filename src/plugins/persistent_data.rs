use parking_lot::{lock_api::RwLockReadGuard, RawRwLock, RwLock};

use super::*;

pub struct PersistentDataPlugin;

impl Plugin for PersistentDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::update);
    }
}

static DATA: OnceCell<RwLock<Data>> = OnceCell::new();
static EDIT_TS: Mutex<Option<f64>> = Mutex::new(None);

#[derive(Default, Clone, PartialEq)]
pub struct Data {
    pub client_settings: ClientSettings,
    pub client_state: ClientState,
}

pub fn pd() -> RwLockReadGuard<'static, RawRwLock, Data> {
    DATA.get().unwrap().read()
}
pub fn pd_mut(f: impl Fn(&mut Data)) {
    let mut new_data = pd().clone();
    f(&mut new_data);
    if !pd().eq(&new_data) {
        *DATA.get().unwrap().write() = new_data;
        *EDIT_TS.lock() = Some(gt().elapsed());
    }
}

pub trait PersistentData:
    Default + Deserialize<'static> + Serialize + Debug + Clone + PartialEq
{
    fn file_name() -> &'static str;
    fn path() -> PathBuf {
        let mut path = home_dir_path();
        path.push(format!("{}.ron", Self::file_name()));
        path
    }
    fn save(&self) {
        let s = to_ron_string(self);
        match std::fs::write(Self::path(), s) {
            Ok(_) => {
                info!("Store {} successful", Self::path().to_string_lossy())
            }
            Err(e) => {
                error!("Store {} error: {e}", Self::path().to_string_lossy())
            }
        }
    }
}

impl PersistentDataPlugin {
    fn load_data<T: PersistentData>() -> T {
        if let Some(data) = std::fs::read_to_string(T::path())
            .ok()
            .and_then(|d| ron::from_str(d.leak()).ok())
        {
            info!("Loaded data from file {}", T::path().to_string_lossy());
            data
        } else {
            error!(
                "Failed to load data from file {}",
                T::path().to_string_lossy()
            );
            let data = T::default();
            data.save();
            data
        }
    }
    pub fn load() {
        let mut data = DATA.get_or_init(|| default()).write();
        data.client_settings = Self::load_data();
        data.client_state = Self::load_data();
    }
    fn save() {
        let pd = pd();
        pd.client_settings.save();
        pd.client_state.save();
    }
    fn update() {
        let ts = *EDIT_TS.lock();
        if let Some(ts) = ts {
            if gt().elapsed() > ts + 1.0 {
                *EDIT_TS.lock() = None;
                Self::save();
            }
        }
    }
}
