use spacetimedb_sdk::identity::Credentials;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, AsRefStr)]
pub enum GameOption {
    Connect,
    Login,
    ForceLogin,
    TestScenariosLoad,
    Table(StdbQuery),
    ActiveRun,
}

impl ToCstr for GameOption {
    fn cstr(&self) -> Cstr {
        match self {
            GameOption::Connect
            | GameOption::Login
            | GameOption::ForceLogin
            | GameOption::TestScenariosLoad
            | GameOption::ActiveRun => self.as_ref().cstr_c(GREEN),
            GameOption::Table(q) => self
                .as_ref()
                .cstr_c(GREEN)
                .push_wrapped_circ(q.cstr())
                .take(),
        }
    }
}

static CURRENTLY_FULFILLING: Mutex<GameOption> = Mutex::new(GameOption::Connect);
pub fn currently_fulfilling() -> GameOption {
    CURRENTLY_FULFILLING.lock().unwrap().clone()
}

impl GameOption {
    pub fn is_fulfilled(&self, world: &World) -> bool {
        match self {
            GameOption::Connect => world.get_resource::<ConnectOption>().is_some(),
            GameOption::Login | GameOption::ForceLogin => {
                world.get_resource::<LoginOption>().is_some()
            }
            GameOption::TestScenariosLoad => world.get_resource::<TestScenarios>().is_some(),
            GameOption::Table(query) => query.is_subscribed(),
            GameOption::ActiveRun => TArenaRun::get_current().is_some(),
        }
    }
    pub fn fulfill(&self, world: &mut World) {
        info!("{} {}", "Start fulfill option:".dimmed(), self.cstr());
        *CURRENTLY_FULFILLING.lock().unwrap() = self.clone();
        match self {
            GameOption::Connect => ConnectOption::fulfill(world),
            GameOption::Login | GameOption::ForceLogin => LoginOption::fulfill(world),
            GameOption::TestScenariosLoad => GameState::TestScenariosLoad.set_next(world),
            GameOption::Table(query) => {
                StdbQuery::subscribe([query.clone()], |world| GameState::proceed(world));
            }
            GameOption::ActiveRun => {
                GameState::Title.proceed_to_target(world);
            }
        }
    }
}

#[derive(Resource)]
pub struct LoginOption {
    pub player: TPlayer,
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

static PLAYER_NAME: Mutex<&'static str> = Mutex::new("");
static PLAYER_ID: Mutex<u64> = Mutex::new(0);
pub fn player_id() -> u64 {
    *PLAYER_ID.lock().unwrap()
}
pub fn user_name() -> &'static str {
    *PLAYER_NAME.lock().unwrap()
}
impl OptionResource for LoginOption {
    fn fulfill(world: &mut World) {
        GameState::Login.set_next(world);
    }
    fn save(self, world: &mut World) {
        *PLAYER_NAME.lock().unwrap() = self.player.name.clone().leak();
        *PLAYER_ID.lock().unwrap() = self.player.id;
        world.insert_resource(self);
    }
}
