use super::*;

#[derive(Default)]
pub struct SoundData {
    pub loops: HashMap<SoundType, geng::SoundEffect>,
}
