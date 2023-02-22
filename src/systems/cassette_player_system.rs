use super::*;

use geng::ui::*;

pub struct CassettePlayerSystem {
    mode: PlayMode,
}

impl CassettePlayerSystem {
    pub fn new(mode: PlayMode) -> Self {
        Self { mode }
    }
}

const REWIND_SPEED: f32 = 5.0;

impl System for CassettePlayerSystem {
    fn update(&mut self, _world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.head = match self.mode {
            PlayMode::Play | PlayMode::Hidden => resources.cassette.head + resources.delta_time,
            PlayMode::Stop => resources.cassette.head,
            PlayMode::Rewind { ts } => (resources.cassette.head
                + (ts - resources.cassette.head) * resources.delta_time * REWIND_SPEED)
                .clamp(0.01, resources.cassette.length()),
        };
        if self.mode == PlayMode::Hidden {
            return;
        }

        if resources.down_keys.contains(&geng::Key::Space) {
            self.mode = match self.mode {
                PlayMode::Play => PlayMode::Stop,
                PlayMode::Stop | PlayMode::Rewind { .. } => PlayMode::Play,
                _ => panic!("Wrong Play Mode"),
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
                    _ => panic!("Wrong Play Mode"),
                }
                .clamp(0.0, resources.cassette.length()),
            };
        }
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        if self.mode == PlayMode::Hidden {
            return Box::new(ui::Void);
        }
        Box::new(
            (
                Text::new(
                    format!("{:.2}", resources.cassette.head),
                    resources.fonts.get_font(0),
                    64.0,
                    Rgba::BLACK,
                ),
                Text::new(
                    format!("Mode {}", self.mode),
                    resources.fonts.get_font(0),
                    64.0,
                    Rgba::BLACK,
                ),
            )
                .column()
                .align(vec2(0.0, 0.0))
                .fixed_size(vec2(200.0, 60.0))
                .uniform_padding(16.0),
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum PlayMode {
    Play,
    Stop,
    Rewind { ts: Time },
    Hidden,
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
                PlayMode::Hidden => "Hidden",
            }
        )
    }
}
