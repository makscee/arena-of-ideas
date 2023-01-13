use super::*;
pub struct VisualQueue {
    pub nodes: Vec<VisualNode>,
    pub persistent_effects: Vec<Box<dyn VisualEffect>>, // effects that are merged into every node
}

impl VisualQueue {
    pub fn new() -> Self {
        Self {
            nodes: default(),
            persistent_effects: default(),
        }
    }

    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, timestamp: Time) {
        let mut node = self.get_node(timestamp);
        node.update(timestamp, &self.persistent_effects);
        node.draw(framebuffer, timestamp, &self.persistent_effects);
    }
    fn get_node(&self, timestamp: Time) -> VisualNode {
        let index = match self
            .nodes
            .binary_search_by_key(&r32(timestamp), |node| r32(node.timestamp))
        {
            Ok(index) => index,
            Err(index) => index,
        };
        self.nodes
            .get(index)
            .clone()
            .expect("Could not find VisualNode for timestamp")
            .clone()
    }
}
