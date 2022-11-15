use std::fmt::format;

use super::*;
use geng::{prelude::itertools::Itertools, Draw2d};

mod card;

pub use card::*;

const TEXT_OCCUPY_SPACE: f32 = 0.6;
const BACKGROUND_COLOR: Rgba<f32> = Rgba::BLACK;
const TEXT_BACKGROUND_COLOR: Rgba<f32> = Rgba {
    r: 0.2,
    g: 0.2,
    b: 0.2,
    a: 1.0,
};
const SELL_BUTTON_FRAME_WIDTH: f32 = 0.01;
const SELL_BUTTON_COLOR: Rgba<f32> = Rgba {
    r: 0.3,
    g: 0.3,
    b: 0.3,
    a: 1.0,
};
const BUTTON_COLOR: Rgba<f32> = Rgba {
    r: 0.0,
    g: 0.7,
    b: 1.0,
    a: 1.0,
};
const BUTTON_HOVER_COLOR: Rgba<f32> = Rgba {
    r: 0.0,
    g: 0.5,
    b: 0.8,
    a: 1.0,
};
const BUTTON_PRESS_COLOR: Rgba<f32> = Rgba {
    r: 0.0,
    g: 0.3,
    b: 0.6,
    a: 1.0,
};
const TEXT_COLOR: Rgba<f32> = Rgba::WHITE;
const CLAN_MAX_SIZE: f32 = 0.15;
const BAR_SIZE: f32 = 0.1;
/// Relative to the framebuffer size
const CLAN_INFO_SIZE: Vec2<f32> = vec2(0.3, 0.2);
/// Relative to the clan info size
const FONT_SIZE: f32 = 0.1;
const CLAN_BACKGROUND_COLOR: Rgba<f32> = Rgba {
    r: 0.3,
    g: 0.3,
    b: 0.3,
    a: 1.0,
};
const CLAN_INFO_BACKGROUND_COLOR: Rgba<f32> = Rgba {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

fn draw_clan_info(
    position: Vec2<f32>,
    description: &str,
    geng: &Geng,
    framebuffer: &mut ugli::Framebuffer,
    camera: &impl geng::AbstractCamera2d,
) {
    let framebuffer_size = framebuffer.size().map(|x| x as f32);
    let info_aabb = AABB::point(position).extend_positive(CLAN_INFO_SIZE * framebuffer_size);
    let dx = (info_aabb.x_max - framebuffer_size.x).max(0.0);
    let dy = (info_aabb.y_max - framebuffer_size.y).max(0.0);
    let info_aabb = info_aabb.translate(-vec2(dx, dy));
    draw_2d::Quad::new(info_aabb, CLAN_INFO_BACKGROUND_COLOR).draw_2d(geng, framebuffer, camera);
    crate::render::draw_text_wrapped(
        &**geng.default_font(),
        description,
        FONT_SIZE * info_aabb.height(),
        info_aabb,
        TEXT_COLOR,
        framebuffer,
        camera,
    );
}

fn draw_rectangle(
    text: impl AsRef<str>,
    aabb: AABB<f32>,
    color: Rgba<f32>,
    geng: &Geng,
    framebuffer: &mut ugli::Framebuffer,
) {
    let camera = &geng::PixelPerfectCamera;
    draw_2d::Quad::new(aabb, color).draw_2d(geng, framebuffer, camera);
    draw_2d::Text::unit(&**geng.default_font(), text, TEXT_COLOR)
        .fit_into(
            AABB::point(aabb.center()).extend_symmetric(aabb.size() * TEXT_OCCUPY_SPACE / 2.0),
        )
        .draw_2d(geng, framebuffer, camera);
}

fn draw_rectangle_frame(
    text: impl AsRef<str>,
    aabb: AABB<f32>,
    width: f32,
    color: Rgba<f32>,
    geng: &Geng,
    framebuffer: &mut ugli::Framebuffer,
) {
    let camera = &geng::PixelPerfectCamera;
    let left_mid = vec2(aabb.x_min, aabb.center().y);
    draw_2d::Chain::new(
        Chain::new(vec![
            left_mid,
            aabb.top_left(),
            aabb.top_right(),
            aabb.bottom_right(),
            aabb.bottom_left(),
            left_mid,
        ]),
        width,
        color,
        0,
    )
    .draw_2d(geng, framebuffer, camera);
    draw_2d::Text::unit(&**geng.default_font(), text, TEXT_COLOR)
        .fit_into(
            AABB::point(aabb.center()).extend_symmetric(aabb.size() * TEXT_OCCUPY_SPACE / 2.0),
        )
        .draw_2d(geng, framebuffer, camera);
}
