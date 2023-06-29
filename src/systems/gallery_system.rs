use super::*;

pub struct GallerySystem;

impl GallerySystem {
    pub fn enter_state(resources: &mut Resources) {
        resources.gallery_data.current_house = 0;
        PanelsSystem::clear(resources);
        Self::open(resources);
    }

    pub fn leave_state(resources: &mut Resources) {
        Self::close(resources);
    }

    pub fn open(resources: &mut Resources) {
        if let Some(panel) = resources.gallery_data.panel.take() {
            PanelsSystem::close_alert(panel, resources);
        }
        let house =
            enum_iterator::all::<HouseName>().collect_vec()[resources.gallery_data.current_house];

        let mut shaders: Vec<Shader> = default();
        for unit in HeroPool::all(resources) {
            let unit_house = unit.house.unwrap();
            if unit_house == house {
                let shader = unit.get_ui_shader(Faction::Team, false, resources);
                shaders.push(shader);
            }
        }
        fn input_handler(
            event: HandleEvent,
            _: legion::Entity,
            _: &mut Shader,
            _: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => GallerySystem::next(resources),
                _ => {}
            }
        }
        resources.gallery_data.panel = PanelsSystem::open_card_list(
            shaders,
            &house.to_string(),
            HousePool::get_color(&house, resources),
            vec2::ZERO,
            6,
            input_handler,
            "Next",
            resources,
        );
    }

    pub fn next(resources: &mut Resources) {
        resources.gallery_data.current_house = (resources.gallery_data.current_house + 1)
            % (enum_iterator::all::<HouseName>().count() - 1);
        // todo: after Test house is removed, do not subtract 1
        Self::open(resources);
    }

    pub fn close(resources: &mut Resources) {
        if let Some(panel) = resources.gallery_data.panel.take() {
            PanelsSystem::close_alert(panel, resources);
        }
    }
}
