use super::*;

#[derive(Default)]
pub struct TapePlayer {
    pub head: Time,
    pub prev_head: Time,
    pub velocity: Time,
    pub need_velocity: Time,
    pub mode: TapePlayMode,
    pub tape: Tape,
}

#[derive(Eq, PartialEq)]
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
