use super::*;

pub struct GallerySystem {
    pub current_heroes: Vec<usize>,
    pub wanted_hero: usize,
    pub paths: Vec<PathBuf>,
    pub need_redraw: bool,
    pub need_card_animation: bool,
    pub is_card: bool,
}

impl GallerySystem {
    pub fn new() -> Self {
        Self {
            current_heroes: vec![1],
            wanted_hero: 0,
            paths: default(),
            need_redraw: true,
            is_card: false,
            need_card_animation: false,
        }
    }
}

const ZOOM_MULTIPLIER: f32 = 1.5;
const UNIT_SPACING: f32 = 3.0;

impl GallerySystem {
    fn redraw_units(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let heroes = self.current_heroes.len();
        self.current_heroes[heroes - 1] = self.wanted_hero;
        resources.cassette.parallel_node.clear();
        WorldSystem::clear_factions(world, hashset! {Faction::Gallery});
        for (ind, template_ind) in self.current_heroes.iter().enumerate() {
            let template_key = self.paths[*template_ind].clone();
            let position = vec2(
                UNIT_SPACING * ind as f32 - (heroes - 1) as f32 * 0.5 * UNIT_SPACING,
                0.0,
            );
            let entity = UnitTemplatesPool::create_unit_entity(
                &template_key,
                resources,
                world,
                Faction::Gallery,
                0,
                position,
            );
            if self.need_card_animation {
                resources
                    .cassette
                    .parallel_node
                    .add_effect(VisualEffect::new_delayed(
                        1.0,
                        resources.cassette.head,
                        VisualEffectType::EntityShaderAnimation {
                            entity: entity,
                            from: hashmap! {
                                "u_card" => ShaderUniform::Float(if self.is_card {0.0} else {1.0})
                            }
                            .into(),
                            to: hashmap! {
                                "u_card" => ShaderUniform::Float(if self.is_card {1.0} else {0.0})
                            }
                            .into(),
                            easing: EasingType::QuartInOut,
                        },
                        -1,
                    ));
            }
            resources
                .cassette
                .parallel_node
                .add_effect(VisualEffect::new(
                    0.0,
                    VisualEffectType::EntityShaderConst {
                        entity,
                        uniforms: hashmap! {
                            "u_card" => ShaderUniform::Float(if self.is_card {1.0} else {0.0})
                        }
                        .into(),
                    },
                    -2,
                ));
        }
        self.need_card_animation = false;
        UnitSystem::draw_all_units_to_cassette_node(
            world,
            &resources.options,
            &resources.status_pool,
            &mut resources.cassette.parallel_node,
            hashset! {Faction::Gallery},
        );
    }
}

impl System for GallerySystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if self.paths.is_empty() {
            self.paths = Vec::from_iter(resources.unit_templates.heroes.keys().map(|p| p.clone()));
        }
        if self.need_redraw {
            self.redraw_units(world, resources);
            self.need_redraw = false;
        }
        if *self.current_heroes.last().unwrap() != self.wanted_hero {
            self.need_redraw = true;
        }

        if resources.down_keys.contains(&geng::Key::Enter) {
            self.current_heroes.push(self.wanted_hero + 1);
        }

        if resources.down_keys.contains(&geng::Key::Left) {
            let length = self.paths.len();
            self.wanted_hero = (self.wanted_hero + length - 1) % length;
        }
        if resources.down_keys.contains(&geng::Key::Right) {
            self.wanted_hero = (self.wanted_hero + 1) % self.paths.len();
        }
        if resources.down_keys.contains(&geng::Key::Down) {
            self.need_redraw = true;
            resources.camera.fov *= ZOOM_MULTIPLIER;
            WorldSystem::set_var(
                world,
                VarName::FieldPosition,
                &Var::Float(
                    WorldSystem::get_var_float(world, &VarName::FieldPosition) * ZOOM_MULTIPLIER,
                ),
            )
        }
        if resources.down_keys.contains(&geng::Key::Up) {
            self.need_redraw = true;
            resources.camera.fov /= ZOOM_MULTIPLIER;
            WorldSystem::set_var(
                world,
                VarName::FieldPosition,
                &Var::Float(
                    WorldSystem::get_var_float(world, &VarName::FieldPosition) / ZOOM_MULTIPLIER,
                ),
            )
        }

        if resources.down_keys.contains(&geng::Key::Space) {
            self.is_card = !self.is_card;
            self.need_card_animation = true;
            self.need_redraw = true;
        }
    }
}
