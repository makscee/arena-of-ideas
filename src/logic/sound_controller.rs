use super::*;

pub struct SoundController {
    pub sounds: Sounds,
}

impl SoundController {
    pub fn new(sounds: Sounds) -> Self {
        Self { sounds }
    }

    pub fn start_music(&mut self) {
        let mut music = self.sounds.get_mut("music_loop.ogg").unwrap();
        music.looped = true;
        music.play();
    }

    pub fn click(&self) {
        self.sounds["click.ogg"].play();
    }

    pub fn buy(&self) {
        self.sounds["shop.ogg"].play();
    }

    pub fn sell(&self) {
        self.sounds["shop.ogg"].play();
    }

    pub fn start(&self) {
        self.sounds["start_game.ogg"].play();
    }

    pub fn win(&self) {
        self.sounds["win_game.ogg"].play();
    }

    pub fn lose(&self) {
        self.sounds["lose_game.ogg"].play();
    }

    pub fn merge(&self) {
        self.sounds["level_up.ogg"].play();
    }
}
