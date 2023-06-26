use super::*;

pub struct GallerySystem {}

impl GallerySystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn enter_state(resources: &mut Resources) {
        let mut shaders: HashMap<HouseName, Vec<Shader>> = default();
        for unit in HeroPool::all(resources) {
            let house = unit.house.unwrap();
            shaders
                .entry(house)
                .or_default()
                .push(unit.get_ui_shader(Faction::Team, resources));
        }
        for (house, shaders) in shaders {
            PanelsSystem::open_card_list(
                shaders,
                &house.to_string(),
                HousePool::get_color(&house, resources),
                vec2::ZERO,
                3,
                resources,
            );
            break;
        }
    }
}

impl System for GallerySystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {}
}
