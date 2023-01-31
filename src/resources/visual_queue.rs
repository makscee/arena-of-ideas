use super::*;

pub struct VisualQueue {
    pub current_ts: Time,
    queue: Vec<VisualNode>,
}

impl Default for VisualQueue {
    fn default() -> Self {
        Self {
            current_ts: default(),
            queue: vec![default()],
        }
    }
}

impl VisualQueue {
    pub fn next_node(&mut self) {
        let node = self.queue.last().unwrap();
        if node.duration <= 0.0 {
            return;
        }
        self.queue.push(VisualNode {
            start: node.start + node.duration,
            duration: 0.0,
            effects: default(),
        });
    }

    pub fn add_effect(&mut self, effect: VisualEffect) {
        let mut node = self.queue.last_mut().unwrap();
        node.duration = node.duration.max(effect.duration);
        node.effects.push(effect);
    }

    pub fn get_shaders(&self) -> Vec<Shader> {
        let node = self.get_node_at_ts(self.current_ts);
        let time = self.current_ts - node.start;
        let mut shaders: Vec<Shader> = default();
        for effect in node.effects.iter() {
            shaders.extend(effect.r#type.process(time / effect.duration));
        }
        shaders
    }

    fn get_node_at_ts(&self, ts: Time) -> &VisualNode {
        let index = match self
            .queue
            .binary_search_by_key(&r32(ts), |node| r32(node.start))
        {
            Ok(index) => index,
            Err(index) => index - 1,
        };
        if let Some(node) = self.queue.get(index) {
            node
        } else {
            &self.queue.last().unwrap()
        }
    }
}

#[derive(Default, Clone)]

struct VisualNode {
    start: Time,
    duration: Time,
    effects: Vec<VisualEffect>,
}
