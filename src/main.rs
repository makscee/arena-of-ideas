use geng::prelude::*;

mod config;

type Health = i32;
type Time = R32;
type Coord = R32;
type Id = i64;

#[derive(geng::Assets)]
pub struct Assets {
    units: config::UnitTemplates,
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

#[derive(Serialize, Deserialize)]
pub enum AttackState {
    None,
    Start { time: Time, target: Id },
    Cooldown { time: Time },
}

#[derive(Serialize, Deserialize)]
pub enum Status {
    Freeze,
    Slow { percent: f32, time: Time },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Suicide,
    FreezeTarget,
    Slow { percent: f32, time: Time },
}

#[derive(Serialize, Deserialize, HasId)]
pub struct Unit {
    pub id: Id,
    pub statuses: Vec<Status>,
    pub faction: Faction,
    pub attack_state: AttackState,
    pub hp: Health,
    pub position: Vec2<Coord>,
    pub speed: Coord,
    pub projectile_speed: Option<Coord>,
    pub attack_radius: Coord,
    pub size: Coord,
    pub attack_damage: Health,
    pub attack_cooldown: Time,
    pub attack_effects: Vec<Effect>,
    pub attack_animation_delay: Time,
    pub move_ai: MoveAi,
    pub target_ai: TargetAi,
    pub color: Color<f32>,
}

impl Unit {
    pub fn new(
        id: Id,
        template: &config::UnitTemplate,
        faction: Faction,
        position: Vec2<Coord>,
    ) -> Self {
        Self {
            id,
            statuses: Vec::new(),
            faction,
            attack_state: AttackState::None,
            hp: template.hp,
            position,
            speed: template.speed,
            projectile_speed: template.projectile_speed,
            attack_radius: template.attack_radius,
            size: template.size,
            attack_damage: template.attack_damage,
            attack_cooldown: template.attack_cooldown,
            attack_animation_delay: template.attack_animation_delay,
            attack_effects: template.attack_effects.clone(),
            move_ai: template.move_ai,
            target_ai: template.target_ai,
            color: template.color,
        }
    }

    pub fn radius(&self) -> Coord {
        self.size / Coord::new(2.0)
    }
}

fn distance_between_units(a: &Unit, b: &Unit) -> Coord {
    (a.position - b.position).len() - a.radius() - b.radius()
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
    pub damage: Health,
}

pub struct RoundState {
    next_id: Id,
    assets: Assets,
    config: config::Config,
    geng: Geng,
    camera: geng::Camera2d,
    units: Collection<Unit>,
    projectiles: Collection<Projectile>,
}

impl RoundState {
    pub fn new(geng: &Geng, assets: Assets) -> Self {
        let config: config::Config =
            serde_json::from_reader(std::fs::File::open(static_path().join("state.json")).unwrap())
                .unwrap();
        let mut next_id = 0;
        let mut units = Collection::new();
        for unit_type in &config.player {
            let template = &assets.units.map[unit_type];
            let unit = Unit::new(
                next_id,
                template,
                Faction::Player,
                vec2(
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                ) * Coord::new(0.01),
            );
            next_id += 1;
            units.insert(unit);
        }
        Self {
            next_id,
            assets,
            config,
            geng: geng.clone(),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 10.0,
            },
            units,
            projectiles: Collection::new(),
        }
    }

    fn process_statuses(&mut self, unit: &mut Unit, delta_time: Time) {
        for status in &mut unit.statuses {
            match status {
                Status::Slow { time, .. } => {
                    *time -= delta_time;
                }
                _ => {}
            }
        }
        unit.statuses.retain(|status| match status {
            Status::Slow { time, .. } => *time > Time::ZERO,
            _ => true,
        });
    }

    fn process_movement(&mut self, unit: &mut Unit, delta_time: Time) {
        if unit
            .statuses
            .iter()
            .any(|status| matches!(status, Status::Freeze))
        {
            return;
        }
        if matches!(unit.attack_state, AttackState::Start { .. }) {
            return;
        }
        let mut target_position = unit.position;
        match unit.move_ai {
            MoveAi::Advance => {
                let closest_enemy = self
                    .units
                    .iter()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len());
                if let Some(closest_enemy) = closest_enemy {
                    if distance_between_units(closest_enemy, &unit) > unit.attack_radius {
                        target_position = closest_enemy.position;
                    }
                }
            }
            _ => todo!(),
        }
        let mut speed = unit.speed;
        for status in &unit.statuses {
            match status {
                Status::Slow { percent, .. } => {
                    speed *= Coord::new(*percent / 100.0);
                }
                _ => {}
            }
        }
        unit.position += (target_position - unit.position).clamp_len(..=speed * delta_time);
    }

    fn process_collisions(&mut self, unit: &mut Unit) {
        for other in &mut self.units {
            let delta_pos = other.position - unit.position;
            let penetration = unit.radius() + other.radius() - delta_pos.len();
            if penetration > Coord::ZERO {
                let dir = delta_pos.normalize_or_zero();
                unit.position -= dir * penetration / Coord::new(2.0);
                other.position += dir * penetration / Coord::new(2.0);
            }
        }
    }
    fn process_targeting(&mut self, unit: &mut Unit) {
        if unit
            .statuses
            .iter()
            .any(|status| matches!(status, Status::Freeze))
        {
            return;
        }
        if let AttackState::None = unit.attack_state {
            let target = match unit.target_ai {
                TargetAi::Closest => self
                    .units
                    .iter_mut()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len()),
                _ => todo!(),
            };
            if let Some(target) = target {
                if distance_between_units(target, &unit) < unit.attack_radius {
                    unit.attack_state = AttackState::Start {
                        time: Time::new(0.0),
                        target: target.id,
                    }
                }
            }
        }
    }
    fn process_attacks(&mut self, unit: &mut Unit, delta_time: Time) {
        if let AttackState::Start { time, target } = &mut unit.attack_state {
            *time += delta_time;
            if *time > unit.attack_animation_delay {
                let target = self.units.get_mut(target);
                unit.attack_state = AttackState::Cooldown {
                    time: Time::new(0.0),
                };
                if let Some(target) = target {
                    if let Some(projectile_speed) = unit.projectile_speed {
                        self.projectiles.insert(Projectile {
                            id: self.next_id,
                            attacker: unit.id,
                            target: target.id,
                            position: unit.position
                                + (target.position - unit.position).normalize() * unit.radius(),
                            speed: projectile_speed,
                            target_position: target.position,
                            effects: unit.attack_effects.clone(),
                            damage: unit.attack_damage,
                        });
                        self.next_id += 1;
                    } else {
                        let effects = unit.attack_effects.clone();
                        let attack_damage = unit.attack_damage;
                        Self::deal_damage(Some(unit), target, &effects, attack_damage);
                    }
                }
            }
        }
    }
    fn process_cooldowns(&mut self, unit: &mut Unit, delta_time: Time) {
        if let AttackState::Cooldown { time } = &mut unit.attack_state {
            *time += delta_time;
            if *time > unit.attack_cooldown {
                unit.attack_state = AttackState::None;
            }
        }
    }
    fn process_projectiles(&mut self, delta_time: Time) {
        // Projectiles
        let mut delete_projectiles = Vec::new();
        for projectile in &mut self.projectiles {
            let mut attacker = self.units.remove(&projectile.attacker);
            if let Some(target) = self.units.get_mut(&projectile.target) {
                projectile.target_position = target.position;
                if (projectile.position - target.position).len() < target.radius() {
                    Self::deal_damage(
                        attacker.as_mut(),
                        target,
                        &projectile.effects,
                        projectile.damage,
                    );
                    delete_projectiles.push(projectile.id);
                }
            }
            if let Some(attacker) = attacker {
                self.units.insert(attacker);
            }
            let max_distance = projectile.speed * delta_time;
            let distance = (projectile.target_position - projectile.position).len();
            if distance < max_distance {
                delete_projectiles.push(projectile.id);
            }
            projectile.position += (projectile.target_position - projectile.position)
                .clamp_len(..=projectile.speed * delta_time);
        }
        for id in delete_projectiles {
            self.projectiles.remove(&id);
        }
    }
    fn check_next_wave(&mut self) {
        if self
            .units
            .iter()
            .filter(|unit| unit.faction != Faction::Player)
            .count()
            == 0
        {
            if !self.config.waves.is_empty() {
                let wave = self.config.waves.remove(0);
                for (spawn_point, units) in wave {
                    let spawn_point = self.config.spawn_points[&spawn_point];
                    for unit_type in units {
                        let template = &self.assets.units.map[&unit_type];
                        let unit = Unit::new(
                            self.next_id,
                            template,
                            Faction::Enemy,
                            spawn_point
                                + vec2(
                                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                                ) * Coord::new(0.01),
                        );
                        self.next_id += 1;
                        self.units.insert(unit);
                    }
                }
            }
        }
    }
    fn deal_damage(
        mut attacker: Option<&mut Unit>,
        target: &mut Unit,
        effects: &[Effect],
        damage: Health,
    ) {
        target.hp -= damage;
        if damage != 0 {
            target
                .statuses
                .retain(|status| !matches!(status, Status::Freeze));
        }
        for effect in effects {
            match effect {
                Effect::FreezeTarget => {
                    target.attack_state = AttackState::None;
                    target.statuses.push(Status::Freeze);
                }
                Effect::Suicide => {
                    if let Some(attacker) = &mut attacker {
                        attacker.hp = -100500;
                    }
                }
                Effect::Slow { percent, time } => {
                    target.statuses.push(Status::Slow {
                        percent: *percent,
                        time: *time,
                    });
                }
            }
        }
    }
}

impl geng::State for RoundState {
    fn update(&mut self, delta_time: f64) {
        let delta_time: Time = Time::new(delta_time as _);
        let ids: Vec<Id> = self.units.ids().copied().collect();
        for unit_id in ids {
            let mut unit = self.units.remove(&unit_id).unwrap();

            self.process_movement(&mut unit, delta_time);
            self.process_collisions(&mut unit);
            self.process_targeting(&mut unit);
            self.process_attacks(&mut unit, delta_time);
            self.process_cooldowns(&mut unit, delta_time);

            self.units.insert(unit);
        }

        self.process_projectiles(delta_time);

        self.units.retain(|unit| unit.hp > 0);

        self.check_next_wave();
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::WHITE), None);
        for unit in &self.units {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Ellipse::circle(
                    unit.position.map(|x| x.as_f32()),
                    unit.radius().as_f32(),
                    {
                        let mut color = unit.color;
                        if unit
                            .statuses
                            .iter()
                            .any(|status| matches!(status, Status::Freeze))
                        {
                            color = Color::CYAN;
                        }
                        if unit
                            .statuses
                            .iter()
                            .any(|status| matches!(status, Status::Slow { .. }))
                        {
                            color = Color::GRAY;
                        }
                        color
                    },
                ),
            );
        }
        for projectile in &self.projectiles {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Ellipse::circle(projectile.position.map(|x| x.as_f32()), 0.1, Color::RED),
            );
        }
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();
    let geng = Geng::new("Arena of Ideas");
    geng::run(
        &geng,
        geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            <Assets as geng::LoadAsset>::load(&geng, static_path().to_str().unwrap()),
            {
                let geng = geng.clone();
                move |assets| {
                    let assets = assets.expect("Failed to load assets");
                    RoundState::new(&geng, assets)
                }
            },
        ),
    );
}
