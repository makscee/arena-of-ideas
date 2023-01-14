use super::*;

// for given timestamp draws all effects
#[derive(Clone)]
pub struct VisualNode {
    pub timestamp: Time, // when this node should be drawn
    pub duration: Time,
    pub effects: Vec<Rc<dyn VisualEffect>>,
    pub model: VisualNodeModel, // any dynamic data that effects can change in their update()
}

impl VisualNode {
    pub fn empty() -> VisualNode {
        let timestamp = 0.0;
        VisualNode {
            timestamp,
            duration: -1.0,
            effects: default(),
            model: VisualNodeModel {
                units: default(),
                timestamp,
            },
        }
    }

    pub fn update(&mut self, timestamp: Time, persistent_effects: &mut Vec<Box<dyn VisualEffect>>) {
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
        render: &ViewRender,
        framebuffer: &mut ugli::Framebuffer,
        timestamp: Time,
        persistent_effects: &Vec<Box<dyn VisualEffect>>,
    ) {
        let t = self.get_t(timestamp);
        persistent_effects
            .iter()
            .for_each(|e| e.draw(render, framebuffer, t));
        self.effects
            .iter()
            .for_each(|e| e.draw(render, framebuffer, t));
    }

    fn get_t(&self, timestamp: Time) -> Time {
        let t = timestamp - self.timestamp;
        if self.duration.is_sign_positive() && (t.is_sign_negative() || t > self.duration) {
            panic!("Tried to draw node with timestamp out of bounds");
        }
        t
    }
}

#[derive(Clone)]
pub struct VisualNodeModel {
    pub units: Collection<UnitRender>,
    pub timestamp: Time,
}

impl VisualNodeModel {
    pub fn new(units: Collection<UnitRender>, timestamp: Time) -> Self {
        Self { units, timestamp }
    }
}
