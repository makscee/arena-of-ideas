use spacetimedb_lib::Identity;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, AsRefStr)]
pub enum GameOption {
    Connect,
    Login,
    ForceLogin,
    TestScenariosLoad,
    ActiveRun,
    ForceTablesSubscribe,
}

impl ToCstr for GameOption {
    fn cstr(&self) -> Cstr {
        match self {
            GameOption::Connect
            | GameOption::Login
            | GameOption::ForceLogin
            | GameOption::TestScenariosLoad
            | GameOption::ForceTablesSubscribe
            | GameOption::ActiveRun => self.as_ref().cstr_c(GREEN),
        }
    }
}

static CURRENTLY_FULFILLING: Mutex<GameOption> = Mutex::new(GameOption::Connect);
pub fn currently_fulfilling() -> GameOption {
    CURRENTLY_FULFILLING.lock().clone()
}

impl GameOption {
    pub fn is_fulfilled(&self, world: &World) -> bool {
        match self {
            GameOption::Connect => world.get_resource::<ConnectOption>().is_some(),
            GameOption::Login | GameOption::ForceLogin => {
                world.get_resource::<LoginOption>().is_some()
            }
            GameOption::TestScenariosLoad => todo!(),
            GameOption::ActiveRun => todo!(),
            GameOption::ForceTablesSubscribe => world.contains_resource::<TablesSubscribeOption>(),
        }
    }
    pub fn fulfill(&self, world: &mut World) {
        info!(
            "{} {}",
            "Start fulfill option:".dimmed(),
            self.cstr().to_colored()
        );
        *CURRENTLY_FULFILLING.lock() = self.clone();
        match self {
            GameOption::Connect => ConnectOption::fulfill(world),
            GameOption::Login | GameOption::ForceLogin => LoginOption::fulfill(world),
            GameOption::TestScenariosLoad => GameState::TestScenariosLoad.set_next(world),
            GameOption::ActiveRun => {
                GameState::Title.proceed_to_target(world);
            }
            GameOption::ForceTablesSubscribe => {
                subscribe_game(|| {
                    GameState::proceed_op();
                });
            }
        }
    }
}

#[derive(Resource, Debug)]
pub struct LoginOption {
    pub player: NPlayer,
}

#[derive(Resource, Clone, Debug)]
pub struct ConnectOption {
    pub identity: Identity,
    pub token: String,
}

#[derive(Resource, Default)]
pub struct TablesSubscribeOption;

pub trait OptionResource: Resource + Sized + Debug {
    fn fulfill(world: &mut World);
    fn save(self, world: &mut World) {
        world.insert_resource(self);
    }
    fn save_op(self) {
        op(|world| {
            self.save(world);
        });
    }
    fn get(world: &World) -> &Self {
        world.get_resource::<Self>().unwrap()
    }
}

impl OptionResource for ConnectOption {
    fn fulfill(world: &mut World) {
        if let Some(login_data) = world.get_resource::<LoginData>() {
            if login_data.id_token.is_some() {
                GameState::Connect.set_next(world);
            } else {
                warn!("Cannot connect: No id_token available. Please login first.");
            }
        } else {
            warn!("Cannot connect: LoginData resource not found.");
        }
    }
}

static PLAYER_NAME: Mutex<&'static str> = Mutex::new("");
static PLAYER_ID: Mutex<u64> = Mutex::new(0);
static PLAYER_IDENTITY: Mutex<Identity> = Mutex::new(Identity::ZERO);
pub fn player_id() -> u64 {
    *PLAYER_ID.lock()
}
pub fn player_name() -> &'static str {
    *PLAYER_NAME.lock()
}
pub fn player_identity() -> Identity {
    *PLAYER_IDENTITY.lock()
}
pub fn player<'a>(ctx: &'a ClientContext) -> NodeResult<&'a NPlayer> {
    ctx.load_ref::<NPlayer>(player_id())
}
pub fn player_mut<'a>(ctx: &'a mut ClientContext) -> NodeResult<Mut<'a, NPlayer>> {
    ctx.load_mut::<NPlayer>(player_id())
}
pub fn save_player_identity(identity: Identity) {
    *PLAYER_IDENTITY.lock() = identity;
}

#[cfg(test)]
pub fn set_player_id_for_test(id: u64) {
    *PLAYER_ID.lock() = id;
}
impl OptionResource for LoginOption {
    fn fulfill(world: &mut World) {
        GameState::Login.set_next(world);
    }
    fn save(self, world: &mut World) {
        *PLAYER_NAME.lock() = self.player.player_name.clone().leak();
        *PLAYER_ID.lock() = self.player.id();
        world.insert_resource(self);
    }
}
