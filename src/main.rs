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
    Spawn { unit_type: config::UnitType },
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
    pub kill_effects: Vec<Effect>,
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
        let mut state = Self {
            next_id: 0,
            assets,
            config: config.clone(),
            geng: geng.clone(),
            camera: geng::Camera2d {
                center: vec2(0.0, 0.0),
                rotation: 0.0,
                fov: 10.0,
            },
            units: Collection::new(),
            projectiles: Collection::new(),
        };
        for unit_type in &config.player {
            let template = state.assets.units.map[unit_type].clone();
            state.spawn_unit(
                &template,
                Faction::Player,
                vec2(
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                ) * Coord::new(0.01),
            );
        }
        state
    }

    pub fn spawn_unit(
        &mut self,
        template: &config::UnitTemplate,
        faction: Faction,
        position: Vec2<Coord>,
    ) {
        let mut unit = Unit {
            id: self.next_id,
            statuses: Vec::new(),
            faction,
            attack_state: AttackState::None,
            hp: template.hp,
            max_hp: template.hp,
            position,
            speed: template.speed,
            projectile_speed: template.projectile_speed,
            attack_radius: template.attack_radius,
            size: template.size,
            attack_damage: template.attack_damage,
            attack_cooldown: template.attack_cooldown,
            attack_animation_delay: template.attack_animation_delay,
            attack_effects: template.attack_effects.clone(),
            kill_effects: template.kill_effects.clone(),
            death_effects: template.death_effects.clone(),
            move_ai: template.move_ai,
            target_ai: template.target_ai,
            color: template.color,
        };
        self.next_id += 1;
        for effect in &template.spawn_effects {
            self.apply_effect(effect, None, &mut unit);
        }
        self.units.insert(unit);
    }

    fn process_statuses(&mut self, unit: &mut Unit, delta_time: Time) {
        for status in &mut unit.statuses {
            match status {
                Status::Slow { time, .. } => {
                    *time -= delta_time;
                }
                Status::Freeze => {
                    unit.attack_state = AttackState::None;
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
                    speed *= Coord::new(1.0 - *percent / 100.0);
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
                let target = self.units.remove(target);
                unit.attack_state = AttackState::Cooldown {
                    time: Time::new(0.0),
                };
                if let Some(mut target) = target {
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
                            kill_effects: unit.kill_effects.clone(),
                            damage: unit.attack_damage,
                        });
                        self.next_id += 1;
                    } else {
                        let effects = unit.attack_effects.clone();
                        let kill_effects = unit.kill_effects.clone();
                        let attack_damage = unit.attack_damage;
                        self.deal_damage(
                            Some(unit),
                            &mut target,
                            &effects,
                            &kill_effects,
                            attack_damage,
                        );
                    }
                    self.units.insert(target);
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
    fn process_deaths(&mut self) {
        for id in self.units.ids().copied().collect::<Vec<Id>>() {
            let mut unit = self.units.remove(&id).unwrap();
            if unit.hp <= 0 {
                let effects = unit.death_effects.clone();
                for effect in effects {
                    self.apply_effect(&effect, None, &mut unit);
                }
            }
            self.units.insert(unit);
        }
        self.units.retain(|unit| unit.hp > 0);
    }
    fn process_projectiles(&mut self, delta_time: Time) {
        // Projectiles
        let mut delete_projectiles = Vec::new();
        for id in self.projectiles.ids().copied().collect::<Vec<Id>>() {
            let mut projectile = self.projectiles.remove(&id).unwrap();

            let mut attacker = self.units.remove(&projectile.attacker);
            if let Some(mut target) = self.units.remove(&projectile.target) {
                projectile.target_position = target.position;
                if (projectile.position - target.position).len() < target.radius() {
                    self.deal_damage(
                        attacker.as_mut(),
                        &mut target,
                        &projectile.effects,
                        &projectile.kill_effects,
                        projectile.damage,
                    );
                    delete_projectiles.push(projectile.id);
                }
                self.units.insert(target);
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

            self.projectiles.insert(projectile);
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
                        let template = self.assets.units.map[&unit_type].clone();
                        self.spawn_unit(
                            &template,
                            Faction::Enemy,
                            spawn_point
                                + vec2(
                                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                                ) * Coord::new(0.01),
                        );
                    }
                }
            }
        }
    }
    fn deal_damage(
        &mut self,
        mut attacker: Option<&mut Unit>,
        target: &mut Unit,
        effects: &[Effect],
        kill_effects: &[Effect],
        mut damage: Health,
    ) {
        damage = min(damage, target.hp);
        if damage != 0 {
            if let Some((index, _)) = target
                .statuses
                .iter()
                .enumerate()
                .find(|(_, status)| matches!(status, Status::Shield))
            {
                damage = 0;
                target.statuses.remove(index);
            }
        }
        if damage != 0 {
            target
                .statuses
                .retain(|status| !matches!(status, Status::Freeze));
        }
        let old_hp = target.hp;
        target.hp -= damage;
        for effect in effects {
            self.apply_effect(effect, attacker.as_deref_mut(), target);
        }
        if old_hp > 0 && target.hp <= 0 {
            for effect in kill_effects {
                self.apply_effect(effect, attacker.as_deref_mut(), target);
            }
        }
    }
    fn apply_effect(&mut self, effect: &Effect, mut caster: Option<&mut Unit>, target: &mut Unit) {
        match effect {
            Effect::AddStatus { status } => {
                target.statuses.push(status.clone());
            }
            Effect::Suicide => {
                if let Some(caster) = &mut caster {
                    caster.hp = -100500;
                }
            }
            Effect::Spawn { unit_type } => {
                let template = self.assets.units.map[unit_type].clone();
                self.spawn_unit(
                    &template,
                    match caster {
                        Some(unit) => unit.faction,
                        None => target.faction,
                    },
                    target.position
                        + vec2(
                            global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                            global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                        ) * Coord::new(0.01),
                )
                // self.new_unit();
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
            self.process_statuses(&mut unit, delta_time);
            self.process_collisions(&mut unit);
            self.process_targeting(&mut unit);
            self.process_attacks(&mut unit, delta_time);
            self.process_cooldowns(&mut unit, delta_time);

            self.units.insert(unit);
        }

        self.process_projectiles(delta_time);

        self.process_deaths();

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
                    unit.radius().as_f32()
                        * match &unit.attack_state {
                            AttackState::Start { time, .. } => {
                                1.0 - 0.25 * (*time / unit.attack_animation_delay).as_f32()
                            }
                            _ => 1.0,
                        },
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
            if unit
                .statuses
                .iter()
                .any(|status| matches!(status, Status::Shield))
            {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(
                        unit.position.map(|x| x.as_f32()),
                        unit.radius().as_f32() * 1.1,
                        Color::rgba(1.0, 1.0, 0.0, 0.5),
                    ),
                );
            }
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::unit(Color::GREEN).transform(
                    Mat3::translate(unit.position.map(|x| x.as_f32()))
                        * Mat3::scale_uniform(unit.radius().as_f32())
                        * Mat3::translate(vec2(0.0, 1.2))
                        * Mat3::scale(0.1 * vec2(10.0 * unit.hp as f32 / unit.max_hp as f32, 1.0)),
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
