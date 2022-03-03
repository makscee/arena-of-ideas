use super::*;

pub struct UnitTemplates {
    pub map: HashMap<String, UnitTemplate>,
}

impl geng::LoadAsset for UnitTemplates {
    fn load(geng: &Geng, path: &str) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let json = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let types: Vec<String> = serde_json::from_str(&json)?;
            let mut map = HashMap::new();
            let base_path = &path[..path.rfind('/').unwrap()];
            for typ in types {
                let template = <UnitTemplate as geng::LoadAsset>::load(
                    &geng,
                    &format!("{}/units/{}.json", base_path, typ),
                );
                map.insert(typ, template.await?);
            }
            Ok(Self { map })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("json");
}

pub type Wave = HashMap<String, Vec<UnitType>>;

#[derive(Deserialize)]
pub struct Config {
    pub player: Vec<UnitType>,
    pub spawn_points: HashMap<String, Vec2<Coord>>,
    pub waves: Vec<Wave>,
}

pub type UnitType = String;

#[derive(Deserialize)]
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
