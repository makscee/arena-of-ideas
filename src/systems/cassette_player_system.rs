use super::*;

use geng::ui::*;

pub struct CassettePlayerSystem {
    mode: PlayMode,
}

impl CassettePlayerSystem {
    pub fn new() -> Self {
        Self {
            mode: PlayMode::Play,
        }
    }
}

const REWIND_SPEED: f32 = 5.0;

impl System for CassettePlayerSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.head = match self.mode {
            PlayMode::Play => resources.cassette.head + resources.delta_time,
            PlayMode::Stop => resources.cassette.head,
            PlayMode::Rewind { ts } => {
                resources.cassette.head
                    + (ts - resources.cassette.head) * resources.delta_time * REWIND_SPEED
            }
        };

        if resources.down_keys.contains(&geng::Key::Space) {
            self.mode = match self.mode {
                PlayMode::Play => PlayMode::Stop,
                PlayMode::Stop | PlayMode::Rewind { .. } => PlayMode::Play,
            };
        }
        if resources.pressed_keys.contains(&geng::Key::Left)
            || resources.pressed_keys.contains(&geng::Key::Right)
        {
            let direction = if resources.pressed_keys.contains(&geng::Key::Right) {
                1.0
            } else {
                -1.0
            };
            self.mode = PlayMode::Rewind {
                ts: match self.mode {
                    PlayMode::Play | PlayMode::Stop => {
                        resources.cassette.head + resources.delta_time * direction
                    }

                    PlayMode::Rewind { ts } => ts + resources.delta_time * direction * REWIND_SPEED,
                }
                .clamp(0.0, resources.cassette.length()),
            };
        }
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        resources: &Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        Box::new(
            (
                Text::new(
                    format!("{:.2}", resources.cassette.head),
                    resources.font.clone(),
                    64.0,
                    Rgba::BLACK,
                ),
                Text::new(
                    format!("Mode {}", self.mode),
                    resources.font.clone(),
                    32.0,
                    Rgba::BLACK,
                ),
            )
                .column()
                .fixed_size(vec2(200.0, 60.0))
                .uniform_padding(16.0)
                .align(vec2(0.0, 1.0)),
        )
    }
}

#[derive(Clone, Copy)]
enum PlayMode {
    Play,
    Stop,
    Rewind { ts: Time },
}

impl Display for PlayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayMode::Play => "Play",
                PlayMode::Stop => "Stop",
                PlayMode::Rewind { .. } => "Rewind",
            }
        )
    }
}
