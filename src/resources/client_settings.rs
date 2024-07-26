use super::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientSettings {
    pub dev_server: (String, String),
    pub prod_server: (String, String),
    pub dev_mode: bool,
}

static CLIENT_SETTINGS: OnceCell<ClientSettings> = OnceCell::new();
const CLIENT_SETTINGS_FILE: &str = "settings.ron";

fn path() -> PathBuf {
    let mut path = home_dir_path();
    path.push(CLIENT_SETTINGS_FILE);
    path
}
pub fn load_client_settings() {
    let cs = if let Some(cs) = std::fs::read_to_string(&path())
        .ok()
        .and_then(|d| ron::from_str::<ClientSettings>(&d).ok())
    {
        cs
    } else {
        let cs = ClientSettings::default();
        match std::fs::write(
            path(),
            to_string_pretty(&cs, PrettyConfig::new())
                .expect("Failed to serialize default client settings"),
        ) {
            Ok(_) => {
                info!("Store successful")
            }
            Err(e) => {
                error!("Store error: {e}")
            }
        }
        cs
    };
    dbg!(&cs);
    CLIENT_SETTINGS
        .set(cs)
        .expect("Failed to set client settings");
}
pub fn client_settings() -> &'static ClientSettings {
    CLIENT_SETTINGS.get().unwrap()
}
pub fn is_dev_mode() -> bool {
    client_settings().dev_mode
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            dev_server: ("http://161.35.88.206:3000".into(), "aoi_dev".into()),
            prod_server: ("http://161.35.88.206:3000".into(), "aoi_prod".into()),
            dev_mode: false,
        }
    }
}
