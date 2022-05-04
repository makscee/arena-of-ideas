use super::*;

pub struct RenderShop {}

impl RenderShop {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, delta_tile: f32) {}
}

pub struct Render {
    geng: Geng,
    camera: geng::Camera2d,
    assets: Rc<Assets>,
}

impl Render {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: ShopConfig) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            camera: geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: 0.5,
            },
        }
    }

    pub fn draw(&mut self, shop: &Shop, render: &RenderShop, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);
    }
}
