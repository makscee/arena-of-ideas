use super::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Faction {
    Player,
    Enemy,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum MoveAi {
    Advance,
    KeepClose,
    Avoid,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum TargetAi {
    Strongest,
    Biggest,
    SwitchOnHit,
    Closest,
    Furthest,
}

#[derive(Serialize, Deserialize)]
pub enum AttackState {
    None,
    Start { time: Time, target: Id },
    Cooldown { time: Time },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Status {
    Freeze,
    Shield,
    Slow { percent: f32, time: Time },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    AddStatus { status: Status },
    Spawn { unit_type: UnitType },
    Suicide,
}

#[derive(Serialize, Deserialize, HasId)]
pub struct Unit {
    pub id: Id,
    pub statuses: Vec<Status>,
    pub faction: Faction,
    pub attack_state: AttackState,
    pub hp: Health,
    pub max_hp: Health,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub projectile_speed: Option<Coord>,
    pub attack_radius: Coord,
    pub size: Coord,
    pub attack_damage: Health,
    pub attack_cooldown: Time,
    pub attack_effects: Vec<Effect>,
    pub kill_effects: Vec<Effect>,
    pub death_effects: Vec<Effect>,
    pub attack_animation_delay: Time,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub color: Color<f32>,
}

impl Unit {
    pub fn radius(&self) -> Coord {
        self.size / Coord::new(2.0)
    }
}

#[derive(HasId)]
pub struct Projectile {
    pub id: Id,
    pub attacker: Id,
    pub target: Id,
    pub target_position: Vec2<Coord>,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub effects: Vec<Effect>,
    pub kill_effects: Vec<Effect>,
    pub damage: Health,
}

pub type UnitType = String;

#[derive(Deserialize, Clone)]
pub struct UnitTemplate {
    pub hp: Health,
    pub speed: Coord,
    pub projectile_speed: Option<Coord>,
    pub attack_radius: Coord,
    pub size: Coord,
    pub attack_damage: Health,
    pub attack_cooldown: Time,
    pub attack_animation_delay: Time,
    #[serde(default)]
    pub attack_effects: Vec<Effect>,
    #[serde(default)]
    pub spawn_effects: Vec<Effect>,
    #[serde(default)]
    pub death_effects: Vec<Effect>,
    #[serde(default)]
    pub kill_effects: Vec<Effect>,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub color: Color<f32>,
}

impl geng::LoadAsset for UnitTemplate {
    fn load(geng: &Geng, path: &str) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            Ok(serde_json::from_str(&json)?)
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

pub type Wave = HashMap<String, Vec<UnitType>>;

#[derive(Deref)]
pub struct UnitTemplates {
    #[deref]
    pub map: HashMap<String, UnitTemplate>,
}
