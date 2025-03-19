use once_cell::sync::OnceCell;

use super::*;

pub const ANIMATION: f32 = 0.4;
pub static GAME_TIMER: OnceCell<Mutex<GameTimer>> = OnceCell::new();

#[derive(Debug)]
pub struct GameTimer {
    pub playback_speed: f32,
    play_head: f32,
    insert_head: f32,
    end: f32,
    batches: Vec<f32>,
    paused: bool,
    last_delta: f32,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            playback_speed: 1.0,
            play_head: 0.0,
            insert_head: 0.0,
            end: 0.0,
            batches: default(),
            paused: false,
            last_delta: 0.0,
        }
    }
}

pub fn gt() -> MutexGuard<'static, GameTimer> {
    GAME_TIMER.get().unwrap().lock()
}

impl GameTimer {
    pub fn ticked(&self, period: f32, offset: f32) -> bool {
        let t = self.play_head + offset;
        (t / period).floor() != ((t - self.last_delta) / period).floor()
    }
    pub fn update(&mut self, delta: f32) {
        let ps = self.playback_speed;
        let paused = self.paused;
        self.advance_play(delta * ps * (!paused as i32 as f32));
    }
    pub fn pause(&mut self, value: bool) -> &mut Self {
        self.paused = value;
        self
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn last_delta(&self) -> f32 {
        self.last_delta
    }
    pub fn play_head(&self) -> f32 {
        self.play_head
    }
    pub fn secs(&self) -> i32 {
        self.play_head as i32
    }
    pub fn play_head_to(&mut self, t: f32) -> &mut Self {
        self.play_head = t;
        self
    }
    pub fn advance_play(&mut self, delta: f32) -> &mut Self {
        self.play_head = (self.play_head + delta).max(0.0);
        self.insert_head = self.insert_head.max(self.play_head);
        self.last_delta = delta;
        self
    }
    pub fn insert_head(&self) -> f32 {
        self.insert_head
    }
    pub fn insert_head_to(&mut self, t: f32) -> &mut Self {
        self.advance_insert(t - self.insert_head);
        self
    }
    pub fn insert_to_end(&mut self) -> &mut Self {
        self.insert_head_to(self.end)
    }
    pub fn advance_insert(&mut self, delta: f32) -> &mut Self {
        self.insert_head += delta;
        self.end = self.end.max(self.insert_head);
        self
    }
    pub fn get_end(&self) -> f32 {
        self.end
    }
    pub fn start_batch(&mut self) -> &mut Self {
        self.batches.push(self.insert_head);
        self
    }
    pub fn end_batch(&mut self) -> &mut Self {
        self.batches.pop();
        self
    }
    pub fn to_batch_start(&mut self) -> &mut Self {
        self.insert_head = *self.batches.last().unwrap();
        self
    }
    pub fn reset(&mut self) -> &mut Self {
        *self = default();
        self
    }
    pub fn skip_to_end(&mut self) -> &mut Self {
        self.play_head = self.end + 5.0;
        self
    }
    pub fn ended(&self) -> bool {
        self.play_head > self.end
    }
}
