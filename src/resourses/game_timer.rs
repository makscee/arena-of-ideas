use std::ops::Neg;

use super::*;

#[derive(Resource, Default)]
pub struct GameTimer {
    play_head: f32,
    insert_head: f32,
    end: f32,
    batches: Vec<f32>,
    paused: bool,
}

impl GameTimer {
    pub fn pause(&mut self, value: bool) -> &mut Self {
        self.paused = value;
        self
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn get_mut(world: &mut World) -> Mut<GameTimer> {
        world.get_resource_mut::<GameTimer>().unwrap()
    }

    pub fn get(world: &World) -> &GameTimer {
        world.get_resource::<GameTimer>().unwrap()
    }

    pub fn play_head(&self) -> f32 {
        self.play_head
    }

    pub fn play_head_to(&mut self, t: f32) -> &mut Self {
        self.play_head = t;
        self
    }

    pub fn advance_play(&mut self, delta: f32) -> &mut Self {
        if self.paused {
            return self;
        }
        self.play_head += delta;
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

    pub fn advance_insert(&mut self, delta: f32) -> &mut Self {
        self.insert_head += delta;
        self.end = self.end.max(self.insert_head);
        self
    }

    pub fn end(&self) -> f32 {
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
        self.play_head = self.end;
        self
    }

    pub fn ended(&self) -> bool {
        self.play_head > self.end
    }
}
