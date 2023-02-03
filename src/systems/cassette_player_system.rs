use super::*;

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

const REWIND_SPEED: f32 = 10.0;

impl System for CassettePlayerSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.head = match self.mode {
            PlayMode::Play => {
                (resources.cassette.head + resources.delta_time).min(resources.cassette.length())
            }
            PlayMode::Stop => resources.cassette.head,
            PlayMode::Rewind { ts } => {
                resources.cassette.head
                    + (ts - resources.cassette.head) * resources.delta_time * REWIND_SPEED
            }
        };

        if let Some(key) = resources.down_key {
            self.mode = match key {
                geng::Key::Space => match self.mode {
                    PlayMode::Play => PlayMode::Stop,
                    PlayMode::Stop | PlayMode::Rewind { .. } => PlayMode::Play,
                },
                geng::Key::Left | geng::Key::Right => {
                    let right = key == geng::Key::Right;
                    match self.mode {
                        PlayMode::Play | PlayMode::Stop => PlayMode::Rewind {
                            ts: resources
                                .cassette
                                .get_skip_ts(resources.cassette.head, right),
                        },
                        PlayMode::Rewind { ts } => PlayMode::Rewind {
                            ts: resources.cassette.get_skip_ts(ts, right),
                        },
                    }
                }
                _ => self.mode,
            }
        }
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
    }
}

#[derive(Clone, Copy)]
enum PlayMode {
    Play,
    Stop,
    Rewind { ts: Time },
}
