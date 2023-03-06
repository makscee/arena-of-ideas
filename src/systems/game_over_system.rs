use super::*;
use geng::ui::*;

pub struct GameOverSystem {
    pub victory: bool,
}

impl GameOverSystem {
    pub fn new() -> Self {
        Self { victory: default() }
    }

    pub fn init(world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.parallel_node.clear();
        <(&EntityComponent, &AttentionComponent)>::query()
            .iter(world)
            .map(|(entity, _)| entity.entity)
            .collect_vec()
            .into_iter()
            .for_each(|entity| {
                world
                    .entry(entity)
                    .unwrap()
                    .remove_component::<AttentionComponent>();
                world
                    .entry(entity)
                    .unwrap()
                    .remove_component::<HoverComponent>();
            });
        UnitSystem::draw_all_units_to_cassette_node(
            world,
            &resources.options,
            &resources.status_pool,
            &resources.houses,
            &mut resources.cassette.parallel_node,
            hashset! {Faction::Team},
        );
    }
}

impl System for GameOverSystem {
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        let restart = Button::new(cx, "Restart");
        if restart.was_clicked() && resources.shop.money > 0 {
            resources.transition_state = GameState::Shop;
            ShopSystem::restart(world, resources);
        }
        Box::new(
            (
                Text::new("Game Over!", resources.fonts.get_font(0), 70.0, Rgba::BLACK),
                Text::new(
                    format!(
                        "{}",
                        match resources.game_won {
                            true => "Victory!".to_string(),
                            false => format!("Defeat! Round #{}", resources.last_round),
                        }
                    ),
                    resources.fonts.get_font(1),
                    70.0,
                    Rgba::BLACK,
                ),
                restart
                    .uniform_padding(16.0)
                    .background_color(Rgba::try_from("#878787").unwrap()),
            )
                .column()
                .flex_align(vec2(Some(1.0), None), vec2(0.5, 1.0))
                .uniform_padding(32.0),
        )
    }

    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {}
}
