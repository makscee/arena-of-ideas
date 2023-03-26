use super::*;
use geng::ui::*;

#[derive(Default)]
pub struct GameOverSystem {
    pub victory: bool,
    pub need_restart: bool,
}

impl GameOverSystem {
    pub fn new() -> Self {
        default()
    }

    pub fn init(world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.clear();
        SlotSystem::clear_world(world);
        let mut node = CassetteNode::default();
        UnitSystem::draw_all_units_to_cassette_node(
            &hashset! {Faction::Team},
            &mut node,
            world,
            resources,
        );
        resources.cassette.render_node = node;
    }
}

impl System for GameOverSystem {
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        let restart = Button::new(cx, "Restart");
        if restart.was_clicked() {
            self.need_restart = true;
        }
        Box::new(
            (
                Text::new("Game Over!", resources.fonts.get_font(0), 70.0, Rgba::BLACK),
                Text::new(
                    format!(
                        "{}",
                        match resources.game_won {
                            true => "Victory!".to_string(),
                            false => format!("Defeat! Floor #{}", resources.last_round),
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

    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if self.need_restart {
            self.need_restart = false;
            resources.current_state = GameState::MainMenu;
            resources.transition_state = GameState::Shop;
            TeamPool::save_team(
                Faction::Team,
                Team::empty(resources.options.player_team_name.clone()),
                resources,
            );
            resources.status_pool.clear_all_active();
            resources.unit_corpses.clear();
            resources.action_queue.clear();
            resources.floors.reset();
            UnitSystem::clear_factions(
                world,
                resources,
                &hashset! {Faction::Light, Faction::Dark, Faction::Shop, Faction::Team, },
            );
        }
    }
}
