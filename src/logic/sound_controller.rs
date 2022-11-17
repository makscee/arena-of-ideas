use super::*;

const ENABLE_SOUND: bool = false;
pub struct SoundController {
    sounds: Sounds,
}

impl SoundController {
    pub fn new(sounds: Sounds) -> Self {
        Self { sounds }
    }

    pub fn start_music(&mut self) {
        self.play_music_loop("music_loop.ogg".to_string())
    }

    pub fn click(&self) {
        self.play_sound("click.ogg".to_string());
    }

    pub fn buy(&self) {
        self.play_sound("shop.ogg".to_string());
    }

    pub fn sell(&self) {
        self.play_sound("shop.ogg".to_string());
    }

    pub fn start(&self) {
        self.play_sound("start_game.ogg".to_string());
    }

    pub fn win(&self) {
        self.play_sound("win_game.ogg".to_string());
    }

    pub fn lose(&self) {
        self.play_sound("lose_game.ogg".to_string());
    }

    pub fn merge(&self) {
        self.play_sound("level_up.ogg".to_string());
    }

    pub fn play_sound(&self, file: String) {
        if !ENABLE_SOUND {
            return;
        }
        self.sounds[&file].play();
    }

    pub fn play_music_loop(&mut self, file: String) {
        if !ENABLE_SOUND {
            return;
        }
        let mut music = self
            .sounds
            .get_mut(&file)
            .expect(&format!("Can't find music file {}", file));
        music.looped = true;
        music.play();
    }
}
