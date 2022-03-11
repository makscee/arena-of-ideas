use super::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Alliance {
    Spawners,
    Assassins,
    Critters,
}

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
pub struct Aura {
    pub distance: Option<Coord>,
    pub alliance: Option<Alliance>, // TODO: Filter
    pub status: Box<Status>,
    pub time: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Status {
    Freeze,
    Stun { time: Time },
    Shield,
    Slow { percent: f32, time: Time },
    Modifier(Modifier),
    Aura(Aura),
}

impl Status {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Freeze => "Freeze",
            Self::Stun { .. } => "Stun",
            Self::Shield => "Shield",
            Self::Slow { .. } => "Slow",
            Self::Aura { .. } => "Aura",
            Self::Modifier(..) => "Modifier",
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
pub struct DamageValue {
    pub absolute: Health,
    pub relative: R32,
}

impl Mul<R32> for DamageValue {
    type Output = Self;
    fn mul(self, rhs: R32) -> Self {
        Self {
            absolute: self.absolute * rhs,
            relative: self.relative * rhs,
        }
    }
}

impl Add<Health> for DamageValue {
    type Output = Self;
    fn add(self, rhs: R32) -> Self {
        Self {
            absolute: self.absolute + rhs,
            relative: self.relative,
        }
    }
}

impl Default for DamageValue {
    fn default() -> Self {
        Self {
            absolute: Health::ZERO,
            relative: R32::ZERO,
        }
    }
}

impl TryFrom<String> for DamageValue {
    type Error = <f32 as std::str::FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.ends_with('%') {
            let percent = R32::new(value[..value.len() - 1].parse()?);
            Ok(Self {
                absolute: Health::ZERO,
                relative: percent,
            })
        } else {
            let value = Health::new(value.parse()?);
            Ok(Self {
                absolute: value,
                relative: R32::ZERO,
            })
        }
    }
}

#[derive(Serialize, Deserialize, HasId, Clone)]
pub struct Unit {
    pub id: Id,
    pub unit_type: UnitType,
    pub spawn_animation_time_left: Option<Time>,
    pub attached_statuses: Vec<Status>,
    pub all_statuses: Vec<Status>,
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
    pub triggers: Vec<UnitTrigger>,
    pub alliances: HashSet<Alliance>,
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
#[serde(default)]
pub struct UnitKillTrigger {
    pub damage_type: Option<DamageType>,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "on", deny_unknown_fields)]
pub enum UnitTrigger {
    Death(Effect),
    Spawn(Effect),
    Kill(UnitKillTrigger),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTemplate {
    pub hp: Health,
    pub spawn_animation_time: Time,
    pub speed: Coord,
    pub size: Coord,
    pub attack: AttackProperties,
    pub triggers: Vec<UnitTrigger>,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub abilities: HashMap<Key, Ability>,
    pub color: Color<f32>,
    pub alliances: HashSet<Alliance>,
}

impl UnitTemplate {
    pub fn walk_effects_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.attack.effect.walk_mut(f);
        for trigger in &mut self.triggers {
            match trigger {
                UnitTrigger::Death(effect) => effect.walk_mut(f),
                UnitTrigger::Spawn(effect) => effect.walk_mut(f),
                UnitTrigger::Kill(trigger) => {
                    trigger.effect.walk_mut(f);
                }
            }
        }
        for ability in self.abilities.values_mut() {
            ability.effect.walk_mut(f);
        }
    }
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
            triggers: default(),
            move_ai: MoveAi::Advance,
            target_ai: TargetAi::Closest,
            abilities: HashMap::new(),
            color: Color::BLACK,
            alliances: default(),
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
