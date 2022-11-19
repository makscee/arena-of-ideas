use std::collections::VecDeque;

use super::*;

#[derive(Clone)]
pub struct Text {
    pub position: Vec2<f32>,
    pub render_position: Vec2<f32>,
    pub time: f32,
    pub text: String,
    pub text_type: TextType,
    pub color: Rgba<f32>,
    pub scale: f32, // animated by update()
    pub size: f32,  // initial size
}

#[derive(Clone)]
pub struct TextBlock {
    position: Vec2<f32>,
    top_texts: VecDeque<Text>,
    bot_texts: VecDeque<Text>,
}

impl Text {
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time * 0.25;
        self.scale = f32::max(1.4 - (self.time * 5.0 - 0.3) * (self.time * 5.0 - 0.3), 1.0);
        if self.time > 0.85 {
            self.color.a = 1.0 - (self.time - 0.85) / 0.15;
        }
        self.render_position += (self.position - self.render_position) * delta_time * 5.0;
    }

    pub fn is_alive(&self) -> bool {
        self.time < 1.0
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

    pub fn top_texts(&self) -> impl Iterator<Item = &Text> {
        self.top_texts.iter()
    }

    pub fn bot_texts(&self) -> impl Iterator<Item = &Text> {
        self.bot_texts.iter()
    }

    pub fn texts(&self) -> impl Iterator<Item = &Text> {
        self.top_texts().chain(self.bot_texts())
    }

    pub fn update(&mut self, delta_time: f32) {
        for text in self.top_texts.iter_mut().chain(&mut self.bot_texts) {
            text.update(delta_time);
        }
        self.top_texts.retain(Text::is_alive);
        self.bot_texts.retain(Text::is_alive);
    }

    pub fn add_text_top(&mut self, text: impl Into<String>, color: Rgba<f32>, text_type: TextType) {
        let dir = vec2(0.0, 1.0);
        Self::add_text(
            self.position + dir * 1.5,
            dir,
            &mut self.top_texts,
            text.into(),
            text_type,
            color,
            0.7,
        )
    }

    pub fn add_text_bottom(
        &mut self,
        text: impl Into<String>,
        color: Rgba<f32>,
        text_type: TextType,
    ) {
        let dir = vec2(0.0, -1.0);
        Self::add_text(
            self.position + dir * 1.5,
            dir,
            &mut self.bot_texts,
            text.into(),
            text_type,
            color,
            1.0,
        )
    }

    fn add_text(
        position: Vec2<f32>,
        direction: Vec2<f32>,
        texts: &mut VecDeque<Text>,
        text: String,
        text_type: TextType,
        color: Rgba<f32>,
        size: f32,
    ) {
        for text in texts.iter_mut() {
            text.position += direction * 0.5;
        }
        texts.push_back(Text {
            position,
            render_position: vec2(position.x, 0.0),
            time: 0.0,
            text,
            text_type,
            color,
            size,
            scale: 1.0,
        });
    }
}
