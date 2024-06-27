use super::*;

lazy_static! {
    static ref GAME_TIMER: Mutex<GameTimer> = Mutex::new(default());
}

#[derive(Default)]
pub struct GameTimer {
    play_head: f32,
    insert_head: f32,
    end: f32,
    batches: Vec<f32>,
    paused: bool,
}

impl GameTimer {
    pub fn update(&mut self, delta: f32) {
        if !self.paused {
            self.advance_play(delta);
        }
    }

    pub fn pause(&mut self, value: bool) -> &mut Self {
        self.paused = value;
        self
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn get() -> MutexGuard<'static, GameTimer> {
        GAME_TIMER.lock().unwrap()
    }

    pub fn play_head(&self) -> f32 {
        self.play_head
    }

    pub fn play_head_to(&mut self, t: f32) -> &mut Self {
        self.play_head = t;
        self
    }

    pub fn advance_play(&mut self, delta: f32) -> &mut Self {
        self.play_head = (self.play_head + delta).max(0.0);
        self.insert_head = self.insert_head.max(self.play_head);
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
