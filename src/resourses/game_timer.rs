use super::*;

#[derive(Resource, Default)]
pub struct GameTimer {
    play_head: f32,
    insert_head: f32,
    end: f32,
    batches: Vec<f32>,
}

impl GameTimer {
    pub fn get_mut(world: &mut World) -> Mut<GameTimer> {
        world.get_resource_mut::<GameTimer>().unwrap()
    }

    pub fn get(world: &World) -> &GameTimer {
        world.get_resource::<GameTimer>().unwrap()
    }

    pub fn advance(&mut self, delta: f32) {
        self.play_head += delta;
    }

    pub fn register_insert(&mut self, end: f32) {
        self.insert_head = end;
        self.end = self.end.max(end);
    }

    pub fn get_insert_t(&self) -> f32 {
        self.insert_head
    }

    pub fn get_t(&self) -> f32 {
        self.play_head
    }

    pub fn start_batch(&mut self) {
        self.batches.push(self.end.max(self.play_head))
    }

    pub fn end_batch(&mut self) {
        self.batches.pop();
    }

    pub fn head_to_batch_start(&mut self) {
        self.insert_head = *self.batches.last().unwrap();
    }

    pub fn reset(&mut self) {
        self.play_head = default();
        self.insert_head = default();
        self.end = default();
        self.batches.clear();
    }
}
