use super::*;

pub struct UnitTemplates {
    map: HashMap<String, UnitTemplate>,
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

type Config = HashMap<Faction, Vec<UnitType>>;

type UnitType = String;

#[derive(Deserialize)]
struct UnitTemplate {
    pub hp: Health,
    pub speed: Coord,
    pub projectile_speed: Option<Coord>,
    pub attack_radius: Coord,
    pub size: Coord,
    pub attack_damage: Health,
    pub attack_cooldown: Time,
    pub attack_animation_delay: Time,
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

impl RoundState {
    pub fn load_state(&mut self) {
        let config: Config =
            serde_json::from_reader(std::fs::File::open(static_path().join("state.json")).unwrap())
                .unwrap();
        for (faction, units) in config {
            for unit in units {
                let template = &self.assets.units.map[&unit];
                let unit = Unit {
                    id: self.next_id,
                    faction,
                    attack_state: AttackState::None,
                    hp: template.hp,
                    position: {
                        let center = match faction {
                            Faction::Player => Vec2::ZERO,
                            Faction::Enemy => vec2(Coord::new(5.0), Coord::new(3.0)),
                        };
                        center
                            + vec2(
                                global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                                global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                            ) * Coord::new(0.01)
                    },
                    speed: template.speed,
                    projectile_speed: template.projectile_speed,
                    attack_radius: template.attack_radius,
                    size: template.size,
                    attack_damage: template.attack_damage,
                    attack_cooldown: template.attack_cooldown,
                    attack_animation_delay: template.attack_animation_delay,
                    move_ai: template.move_ai,
                    target_ai: template.target_ai,
                    color: template.color,
                };
                self.next_id += 1;
                self.units.insert(unit);
            }
        }
    }
}
