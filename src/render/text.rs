use std::collections::VecDeque;

use super::*;

#[derive(Clone)]
pub struct Text {
    pub position: Vec2<f32>,
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

impl Text {
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time * 0.5;
        self.scale = f32::max(1.4 - (self.time * 1.5 - 0.3) * (self.time * 1.3 - 0.3), 1.0);
        if self.time > 0.7 {
            self.color.a = 1.0 - (self.time - 0.7) / 0.3;
        }
    }

    pub fn is_alive(&self) -> bool {
        self.time < 1.0
    }
}

impl RenderModel {
    pub(super) fn add_text_random(
        &mut self,
        position: Vec2<f32>,
        text: impl Into<String>,
        color: Color<f32>,
    ) {
        self.texts.push(Text {
            position,
            time: 0.0,
            text: text.into(),
            color,
            scale: 1.0,
        });
    }
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
            text.update(delta_time);
        }
        self.top_texts.retain(Text::is_alive);
        self.bot_texts.retain(Text::is_alive);
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
            time: 0.0,
            text,
            color,
            scale,
        });
    }
}
