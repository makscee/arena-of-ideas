use std::collections::VecDeque;

use super::*;

#[derive(Clone)]
pub struct Text {
    pub position: Vec2<f32>,
    pub velocity: Vec2<f32>,
    pub time: f32,
    pub text: String,
    pub color: Color<f32>,
    pub scale: f32,
}

#[derive(Clone)]
pub struct TextBlock {
    position: Vec2<f32>,
    top_texts: VecDeque<Text>,
    bot_texts: VecDeque<Text>,
}

impl TextBlock {
    pub fn new(position: Vec2<f32>) -> Self {
        Self {
            position,
            top_texts: VecDeque::new(),
            bot_texts: VecDeque::new(),
        }
    }

    pub fn texts(&self) -> impl Iterator<Item = &Text> {
        self.top_texts.iter().chain(&self.bot_texts)
    }

    pub fn update(&mut self, delta_time: f32) {
        for text in self.top_texts.iter_mut().chain(&mut self.bot_texts) {
            text.time += delta_time * 0.8;
            text.position += text.velocity * delta_time;
            text.scale = 1.0 - text.time;
        }
        let is_alive = |text: &Text| text.time < 1.0;
        self.top_texts.retain(is_alive);
        self.bot_texts.retain(is_alive);
    }

    pub fn add_text_top(&mut self, text: impl Into<String>, color: Color<f32>) {
        let dir = vec2(0.0, 1.0);
        Self::add_text(
            self.position + dir * 0.5,
            dir,
            &mut self.top_texts,
            text.into(),
            color,
            1.0,
        )
    }

    pub fn add_text_bottom(&mut self, text: impl Into<String>, color: Color<f32>) {
        let dir = vec2(0.0, -1.0);
        Self::add_text(
            self.position + dir * 0.5,
            dir,
            &mut self.bot_texts,
            text.into(),
            color,
            1.0,
        )
    }

    fn add_text(
        position: Vec2<f32>,
        direction: Vec2<f32>,
        texts: &mut VecDeque<Text>,
        text: String,
        color: Color<f32>,
        scale: f32,
    ) {
        for text in texts.iter_mut() {
            text.position += direction * 0.5;
        }
        texts.push_back(Text {
            position,
            velocity: Vec2::ZERO,
            time: 0.0,
            text,
            color,
            scale,
        });
    }
}
