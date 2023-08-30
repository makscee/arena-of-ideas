use super::*;

#[derive(Resource, Default)]
pub struct GameTimer {
    t: f32,
    end: f32,
    max_end: f32,
    saved_end: f32,
}

impl GameTimer {
    pub fn advance(&mut self, delta: f32) {
        self.t += delta;
    }

    pub fn get_end(&self) -> f32 {
        self.end.max(self.t)
    }

    pub fn get_t(&self) -> f32 {
        self.t
    }

    pub fn update_end(&mut self, t: f32) {
        self.end = t.max(self.end);
        self.max_end = self.end.max(self.max_end);
    }

    pub fn save_end(&mut self) {
        self.saved_end = self.end;
    }

    pub fn return_to_saved_end(&mut self) {
        self.end = self.saved_end;
    }

    pub fn return_to_max_end(&mut self) {
        self.end = self.max_end;
    }
}
