use super::*;

pub struct MouseSystem {}

impl MouseSystem {
    pub fn new() -> Self {
        Self {}
    }

    fn get_hovered_unit(world: &legion::World, resources: &Resources) -> Option<legion::Entity> {
        <(
            &PositionComponent,
            &RadiusComponent,
            &EntityComponent,
            &UnitComponent,
        )>::query()
        .iter(world)
        .find_map(|(position, radius, entity, _)| {
            if (resources.mouse_pos - position.0).len() < radius.0 {
                Some(entity.entity)
            } else {
                None
            }
        })
    }

    fn handle_hover(
        world: &mut legion::World,
        resources: &mut Resources,
        hovered: Option<legion::Entity>,
    ) {
        if let Some(hovered) = hovered {
            if let Some(old_hovered) = resources.hovered_entity {
                if old_hovered == hovered {
                    return;
                }
            }
            world
                .entry(hovered)
                .unwrap()
                .add_component(HoverComponent {});
        }
        if resources.hovered_entity != hovered {
            resources.hovered_entity.and_then(|entity| {
                world.entry(entity).and_then(|mut entry| {
                    entry.remove_component::<HoverComponent>();
                    Some(())
                })
            });
            resources.hovered_entity = hovered;
        }
    }

    fn handle_drag(
        world: &mut legion::World,
        resources: &mut Resources,
        hovered: Option<legion::Entity>,
    ) {
        if resources
            .down_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            if let Some(dragged) = hovered {
                resources.dragged_entity = Some(dragged);
                world
                    .entry(dragged)
                    .unwrap()
                    .add_component(DragComponent {});
            }
        }
        if resources.dragged_entity.is_some()
            && !resources
                .pressed_mouse_buttons
                .contains(&geng::MouseButton::Left)
        {
            world
                .entry(resources.dragged_entity.unwrap())
                .unwrap()
                .remove_component::<DragComponent>();
            resources.dragged_entity = None;
        }
        if let Some(dragged) = resources.dragged_entity {
            world
                .entry(dragged)
                .unwrap()
                .get_component_mut::<PositionComponent>()
                .unwrap()
                .0 = resources.mouse_pos;
        }
    }

    fn update_attention(world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &mut AttentionComponent)>::query()
            .filter(component::<DragComponent>() | component::<HoverComponent>())
            .iter_mut(world)
            .for_each(|(unit, attention)| {
                attention.ts = (attention.ts + resources.delta_time).min(1.0)
            });
        <(&UnitComponent, &mut AttentionComponent)>::query()
            .filter(!component::<DragComponent>() & !component::<HoverComponent>())
            .iter_mut(world)
            .for_each(|(unit, attention)| {
                attention.ts = (attention.ts - resources.delta_time).max(0.0);
            });
    }
}

impl System for MouseSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let hovered = Self::get_hovered_unit(world, resources);
        Self::handle_hover(world, resources, hovered);
        Self::handle_drag(world, resources, hovered);
        Self::update_attention(world, resources);
    }
}
