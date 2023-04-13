use super::*;

#[derive(Default)]
pub struct TapePlayer {
    pub head: Time,
    pub velocity: Time,
    pub mode: TapePlayMode,
    pub tape: Tape,
}

pub enum TapePlayMode {
    Play,
    Stop,
}

impl Default for TapePlayMode {
    fn default() -> Self {
        Self::Play
    }
}

impl TapePlayer {
    pub fn clear(&mut self) {
        *self = default();
    }
}