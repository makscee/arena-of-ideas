use super::*;
pub struct VisualQueue {
    pub nodes: Vec<VisualNode>,
    pub persistent_effects: Vec<Box<dyn VisualEffect>>, // effects that are merged into every node
    pub current_node: VisualNode,
}

impl VisualQueue {
    pub fn new() -> Self {
        Self {
            nodes: default(),
            persistent_effects: default(),
            current_node: VisualNode::empty(),
        }
    }

    pub fn update(&mut self, timestamp: Time) {
        self.current_node = self.get_node(timestamp);
        self.current_node
            .update(timestamp, &mut self.persistent_effects);
    }

    pub fn draw(&self, render: &ViewRender, framebuffer: &mut ugli::Framebuffer, timestamp: Time) {
        self.current_node
            .draw(render, framebuffer, timestamp, &self.persistent_effects);
    }
    fn get_node(&self, timestamp: Time) -> VisualNode {
        let index = match self
            .nodes
            .binary_search_by_key(&r32(timestamp), |node| r32(node.timestamp))
        {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        if let Some(node) = self.nodes.get(index) {
            node.clone()
        } else {
            VisualNode::empty()
        }
    }

    pub fn push_node(&mut self, model: VisualNodeModel, effects: Vec<Rc<dyn VisualEffect>>) {
        let duration = effects
            .iter()
            .map(|e| e.get_duration())
            .max_by(|x, y| x.partial_cmp(&y).unwrap())
            .unwrap();
        let node = VisualNode {
            timestamp: self.nodes.last().map_or(0.0, |n| n.timestamp + n.duration),
            duration,
            effects,
            model,
        };
        debug!(
            "Add node on timestamp: {} duration: {}",
            node.timestamp, node.duration
        );
        self.nodes.push(node);
    }
}
