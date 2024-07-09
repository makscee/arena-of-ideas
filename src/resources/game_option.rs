use spacetimedb_sdk::identity::Credentials;

use super::*;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq)]
pub enum GameOption {
    Connect,
    Login,
    ForceLogin,
}

static CURRENTLY_FULFILLING: Mutex<GameOption> = Mutex::new(GameOption::Connect);
pub fn currently_fulfilling() -> GameOption {
    *CURRENTLY_FULFILLING.lock().unwrap()
}

impl GameOption {
    pub fn is_fulfilled(self, world: &World) -> bool {
        match self {
            GameOption::Connect => world.get_resource::<ConnectOption>().is_some(),
            GameOption::Login | GameOption::ForceLogin => {
                world.get_resource::<LoginOption>().is_some()
            }
        }
    }
    pub fn fulfill(self, world: &mut World) {
        info!("Start fulfill option: {self}");
        *CURRENTLY_FULFILLING.lock().unwrap() = self;
        match self {
            GameOption::Connect => ConnectOption::fulfill(world),
            GameOption::Login => LoginOption::fulfill(world),
            GameOption::ForceLogin => LoginOption::fulfill(world),
        }
    }
}

#[derive(Resource)]
pub struct LoginOption {
    pub user: User,
}

#[derive(Resource, Clone)]
pub struct ConnectOption {
    pub creds: Credentials,
}

pub trait OptionResource: Resource + Sized {
    fn fulfill(world: &mut World);
    fn save(self, world: &mut World) {
        world.insert_resource(self);
    }
    fn get(world: &World) -> &Self {
        world.get_resource::<Self>().unwrap()
    }
}

impl OptionResource for ConnectOption {
    fn fulfill(world: &mut World) {
        GameState::Connect.set_next(world);
    }
}

static USER_NAME: Mutex<&'static str> = Mutex::new("");
static USER_ID: Mutex<u64> = Mutex::new(0);
pub fn user_id() -> u64 {
    *USER_ID.lock().unwrap()
}
pub fn user_name() -> &'static str {
    *USER_NAME.lock().unwrap()
}
impl OptionResource for LoginOption {
    fn fulfill(world: &mut World) {
        GameState::Login.set_next(world);
    }
    fn save(self, world: &mut World) {
        *USER_NAME.lock().unwrap() = self.user.name.clone().leak();
        *USER_ID.lock().unwrap() = self.user.id;
        world.insert_resource(self);
    }
}