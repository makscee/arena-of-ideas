use super::*;

#[derive(Clone)]
pub struct VisualNode {
    pub timestamp: Time,
    pub duration: Time,
    pub effects: Vec<Rc<dyn VisualEffect>>,
    pub model: VisualNodeModel,
}

impl VisualNode {
    pub fn update(&mut self, timestamp: Time, persistent_effects: &Vec<Box<dyn VisualEffect>>) {
        let t = self.get_t(timestamp);
        persistent_effects
            .iter()
            .for_each(|e| e.update(&mut self.model, t));
        self.effects
            .iter()
            .for_each(|e| e.update(&mut self.model, t));
    }

    pub fn draw(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        timestamp: Time,
        persistent_effects: &Vec<Box<dyn VisualEffect>>,
    ) {
        let t = self.get_t(timestamp);
        persistent_effects
            .iter()
            .for_each(|e| e.draw(framebuffer, t));
        self.effects.iter().for_each(|e| e.draw(framebuffer, t));
    }

    fn get_t(&self, timestamp: Time) -> Time {
        let t = timestamp - self.timestamp;
        if t < 0.0 || t > self.duration {
            panic!("Tried to draw node with timestamp out of bounds");
        }
        t
    }
}

#[derive(Clone)]
pub struct VisualNodeModel {
    pub units: Collection<UnitRender>,
}
