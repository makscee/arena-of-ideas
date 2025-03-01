use spacetimedb_lib::Identity;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, AsRefStr)]
pub enum GameOption {
    Connect,
    Login,
    ForceLogin,
    TestScenariosLoad,
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
        }
    }
}

#[derive(Resource, Debug)]
pub struct LoginOption {
    pub player: Player,
}

#[derive(Resource, Clone, Debug)]
pub struct ConnectOption {
    pub identity: Identity,
    pub token: String,
}

pub trait OptionResource: Resource + Sized + Debug {
    fn fulfill(world: &mut World);
    fn save(self, world: &mut World) {
        world.insert_resource(self);
    }
    fn save_op(self) {
        OperationsPlugin::add(|world| {
            self.save(world);
        });
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
static PLAYER_IDENTITY: Mutex<Identity> = Mutex::new(Identity::ZERO);
static PLAYER_ENTITY: Mutex<Entity> = Mutex::new(Entity::PLACEHOLDER);
pub fn player_id() -> u64 {
    *PLAYER_ID.lock()
}
pub fn player_name() -> &'static str {
    *PLAYER_NAME.lock()
}
pub fn player_identity() -> Identity {
    *PLAYER_IDENTITY.lock()
}
pub fn player_entity() -> Entity {
    *PLAYER_ENTITY.lock()
}
pub fn player(world: &World) -> Result<&Player, ExpressionError> {
    Player::get(player_entity(), world).to_e("Player not found")
}
pub fn save_player_identity(identity: Identity) {
    *PLAYER_IDENTITY.lock() = identity;
}
pub fn save_player_entity(entity: Entity) {
    *PLAYER_ENTITY.lock() = entity;
}
impl OptionResource for LoginOption {
    fn fulfill(world: &mut World) {
        GameState::Login.set_next(world);
    }
    fn save(self, world: &mut World) {
        *PLAYER_NAME.lock() = self.player.name.clone().leak();
        *PLAYER_ID.lock() = self.player.id();
        world.insert_resource(self);
    }
}
