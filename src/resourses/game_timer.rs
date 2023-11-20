use super::*;

#[derive(Resource, Default)]
pub struct GameTimer {
    play_head: f32,
    insert_head: f32,
    end: f32,
    batches: Vec<f32>,
    paused: bool,
    save: f32,
}

impl GameTimer {
    pub fn pause(&mut self, value: bool) -> &mut Self {
        self.paused = value;
        self
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn save(&mut self) -> &mut Self {
        self.save = self.play_head;
        self
    }

    pub fn clear_save(&mut self) -> &mut Self {
        self.save = 0.0;
        self
    }

    pub fn head_to_save(&mut self) -> &mut Self {
        self.set_t(self.save);
        self
    }

    pub fn get_mut(world: &mut World) -> Mut<GameTimer> {
        world.get_resource_mut::<GameTimer>().unwrap()
    }

    pub fn get(world: &World) -> &GameTimer {
        world.get_resource::<GameTimer>().unwrap()
    }

    pub fn advance(&mut self, delta: f32) -> &mut Self {
        if self.paused {
            return self;
        }
        self.play_head += delta;
        self
    }

    pub fn advance_end(&mut self, delta: f32) -> &mut Self {
        if self.end < self.play_head {
            self.end = self.play_head;
        }
        self.end += delta;
        self
    }

    pub fn register_insert(&mut self, end: f32) -> &mut Self {
        self.insert_head = end;
        // self.end = self.end.max(end);
        self
    }

    pub fn get_insert_t(&self) -> f32 {
        self.insert_head
    }

    pub fn set_insert_t(&mut self, t: f32) -> &mut Self {
        self.insert_head = t;
        self
    }

    pub fn insert_to_end(&mut self) -> &mut Self {
        self.insert_head = self.end;
        self
    }

    pub fn get_t(&self) -> f32 {
        self.play_head
    }

    pub fn set_t(&mut self, t: f32) -> &mut Self {
        self.play_head = t;
        self
    }

    pub fn start_batch(&mut self) -> &mut Self {
        self.batches.push(self.end.max(self.play_head));
        self
    }

    pub fn end_batch(&mut self) -> &mut Self {
        self.batches.pop();
        self
    }

    pub fn head_to_batch_start(&mut self) -> &mut Self {
        self.insert_head = *self.batches.last().unwrap();
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.play_head = default();
        self.insert_head = default();
        self.end = default();
        self.batches.clear();
        self
    }

    pub fn end(&self) -> f32 {
        self.end
    }

    pub fn ended(&self) -> bool {
        self.play_head > self.end
    }
}
