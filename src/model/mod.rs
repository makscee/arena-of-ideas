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

#[derive(Serialize, Deserialize, Clone)]
pub enum AttackState {
    None,
    Start { time: Time, target: Id },
    Cooldown { time: Time },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Status {
    Freeze,
    Stun { time: Time },
    Shield,
    Slow { percent: f32, time: Time },
}

impl Status {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Freeze => "Freeze",
            Self::Stun { .. } => "Stun",
            Self::Shield => "Shield",
            Self::Slow { .. } => "Slow",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TargetFilter {
    All,
    Allies,
    Enemies,
}

#[derive(Debug, Serialize, Deserialize, Clone, HasId)]
pub struct TimeBomb {
    pub id: Id,
    pub position: Vec2<Coord>,
    pub time: Time,
    pub caster: Option<Id>,
    pub effect: Effect,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(try_from = "String")]
pub enum DamageValue {
    Absolute(Health),
    /// Some percent from hp
    Relative(R32),
}

impl Default for DamageValue {
    fn default() -> Self {
        Self::Absolute(Health::ZERO)
    }
}

impl TryFrom<String> for DamageValue {
    type Error = <f32 as std::str::FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.ends_with('%') {
            let percent = R32::new(value[..value.len() - 1].parse()?);
            Ok(Self::Relative(percent))
        } else {
            let value = Health::new(value.parse()?);
            Ok(Self::Absolute(value))
        }
    }
}

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub unit_type: UnitType,
    pub spawn_animation_time_left: Option<Time>,
    pub statuses: Vec<Status>,
    pub faction: Faction,
    pub attack_state: AttackState,
    pub hp: Health,
    pub max_hp: Health,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub attack: AttackProperties,
    pub size: Coord,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub color: Color<f32>,
    pub ability_cooldown: Option<Time>,
    pub on: UnitTriggers,
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
    pub effect: Effect,
}

pub type UnitType = String;

pub type Key = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Ability {
    pub effect: Effect,
    pub cooldown: Time,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttackProperties {
    pub cooldown: Time,
    pub animation_delay: Time,
    pub radius: Coord,
    #[serde(default)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct UnitKillTrigger {
    pub damage_type: Option<DamageType>,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTriggers {
    pub death: Effect,
    pub spawn: Effect,
    pub kill: UnitKillTrigger,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTemplate {
    pub hp: Health,
    pub spawn_animation_time: Time,
    pub speed: Coord,
    pub size: Coord,
    pub attack: AttackProperties,
    pub on: UnitTriggers,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub abilities: HashMap<Key, Ability>,
    pub color: Color<f32>,
}

impl Default for UnitTemplate {
    fn default() -> Self {
        Self {
            hp: Health::new(1.0),
            spawn_animation_time: Time::new(0.0),
            speed: Coord::new(1.0),
            size: Coord::new(1.0),
            attack: AttackProperties {
                radius: Coord::new(1.0),
                cooldown: Time::new(1.0),
                animation_delay: Time::new(1.0),
                effect: default(),
            },
            on: default(),
            move_ai: MoveAi::Advance,
            target_ai: TargetAi::Closest,
            abilities: HashMap::new(),
            color: Color::BLACK,
        }
    }
}

impl geng::LoadAsset for UnitTemplate {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
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

#[derive(Deref, Clone)]
pub struct UnitTemplates {
    #[deref]
    pub map: HashMap<String, UnitTemplate>,
}

pub struct Model {
    pub next_id: Id,
    pub units: Collection<Unit>,
    pub spawning_units: Collection<Unit>,
    pub dead_units: Collection<Unit>,
    pub projectiles: Collection<Projectile>,
    pub time_bombs: Collection<TimeBomb>,
    pub dead_time_bombs: Collection<TimeBomb>,
    pub config: Config,
    pub unit_templates: UnitTemplates,
}

impl Model {
    pub fn new(config: Config, unit_templates: UnitTemplates) -> Self {
        Self {
            next_id: 0,
            units: Collection::new(),
            spawning_units: Collection::new(),
            dead_units: Collection::new(),
            projectiles: Collection::new(),
            time_bombs: Collection::new(),
            dead_time_bombs: Collection::new(),
            config,
            unit_templates,
        }
    }
}
