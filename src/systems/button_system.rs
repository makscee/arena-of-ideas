use super::*;

pub struct ButtonSystem {}

impl ButtonSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_button(
        world: &mut legion::World,
        world_entity: legion::Entity,
        resources: &Resources,
        icon: &Image,
        action: fn(legion::Entity, &mut Resources, &mut legion::World, ButtonState),
    ) {
        let entity = world.push((
            ButtonComponent::new(action),
            AreaComponent {
                r#type: AreaType::Rectangle {
                    size: vec2(1.0, 1.0),
                },
                position: BATTLEFIELD_POSITION + vec2(0.0, 3.0),
            },
            InputComponent {
                hovered: Some(default()),
                dragged: None,
                pressed: Some(default()),
                clicked: Some(default()),
            },
            resources
                .options
                .shaders
                .icon
                .clone()
                .set_uniform("u_texture", ShaderUniform::Texture(icon.clone()))
                .set_uniform(
                    "u_icon_color",
                    ShaderUniform::Color(Rgba::try_from("#ff7904").unwrap()),
                ),
        ));
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(EntityComponent { entity });
        entry.add_component(Context {
            owner: entity,
            target: entity,
            parent: Some(world_entity),
            vars: default(),
        });
    }

    pub fn change_icon(entity: legion::Entity, world: &mut legion::World, icon: &Image) {
        Self::change_uniform(
            entity,
            world,
            "u_texture".to_string(),
            ShaderUniform::Texture(icon.clone()),
        );
    }

    pub fn change_icon_color(entity: legion::Entity, world: &mut legion::World, color: Rgba<f32>) {
        Self::change_uniform(
            entity,
            world,
            "u_icon_color".to_string(),
            ShaderUniform::Color(color),
        );
    }

    pub fn change_uniform(
        entity: legion::Entity,
        world: &mut legion::World,
        key: String,
        value: ShaderUniform,
    ) {
        let mut entry = world.entry(entity).unwrap();
        let uniforms = &mut entry
            .get_component_mut::<Shader>()
            .unwrap()
            .parameters
            .uniforms;
        uniforms.insert(key, value);
    }
}

impl System for ButtonSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        <(&ButtonComponent, &InputComponent, &EntityComponent)>::query()
            .iter(world)
            .map(|(button, input, entity)| (button.clone(), input.clone(), entity.entity))
            .collect_vec()
            .into_iter()
            .for_each(|(button, input, entity)| {
                if input.is_clicked(resources.global_time) {
                    button.click(entity, resources, world);
                }
                if input.is_pressed(resources.global_time) {
                    button.press(
                        entity,
                        resources,
                        world,
                        resources.global_time - input.pressed.unwrap().1,
                    );
                }
            });
    }
}
