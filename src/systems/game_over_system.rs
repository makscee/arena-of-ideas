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
        let mut node = Node::default();
        UnitSystem::draw_all_units_to_node(&hashset! {Faction::Team}, &mut node, world, resources);
        resources.tape_player.tape.persistent_node = node;
    }
}

impl System for GameOverSystem {
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        _: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        let restart = Button::new(cx, "Restart");
        if restart.was_clicked() {
            self.need_restart = true;
        }
        Box::new(
            (
                Text::new("Game Over!", resources.fonts.get_font(1), 50.0, Rgba::BLACK),
                Text::new(
                    format!(
                        "{}",
                        match resources.battle_data.last_score > 0 {
                            true => "Victory!".to_string(),
                            false => format!("Defeat! Level #{}", resources.battle_data.last_round),
                        }
                    ),
                    resources.fonts.get_font(0),
                    70.0,
                    Rgba::BLACK,
                ),
                Text::new(
                    format!("Total score: {}", resources.battle_data.total_score),
                    resources.fonts.get_font(0),
                    90.0,
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
            Game::restart(world, resources);
        }
    }
}
