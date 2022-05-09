use super::*;

mod card;
mod layout;

pub use card::*;
pub use layout::*;

pub struct RenderShop {
    pub layout: ShopLayout,
}

impl RenderShop {
    pub fn new(
        screen_size: Vec2<f32>,
        shop_cards: usize,
        party_cards: usize,
        inventory_cards: usize,
    ) -> Self {
        Self {
            layout: ShopLayout::new(screen_size, shop_cards, party_cards, inventory_cards),
        }
    }

    pub fn update(&mut self, delta_tile: f32) {}
}

pub struct Render {
    geng: Geng,
    camera: geng::Camera2d,
    assets: Rc<Assets>,
    card_render: CardRender,
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
            card_render: CardRender::new(geng, assets),
        }
    }

    pub fn draw(
        &mut self,
        shop: &Shop,
        render: &RenderShop,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);

        for (index, card) in shop
            .shop
            .iter()
            .enumerate()
            .filter_map(|(i, card)| card.as_ref().map(|card| (i, card)))
        {
            let layout = render
                .layout
                .shop_cards
                .get(index)
                .expect("Invalid shop layout");
            self.card_render.draw(*layout, card, game_time, framebuffer);
        }

        for (index, card) in shop
            .party
            .iter()
            .enumerate()
            .filter_map(|(i, card)| card.as_ref().map(|card| (i, card)))
        {
            let layout = render
                .layout
                .party_cards
                .get(index)
                .expect("Invalid party layout");
            self.card_render.draw(*layout, card, game_time, framebuffer);
        }

        for (index, card) in shop
            .inventory
            .iter()
            .enumerate()
            .filter_map(|(i, card)| card.as_ref().map(|card| (i, card)))
        {
            let layout = render
                .layout
                .inventory_cards
                .get(index)
                .expect("Invalid inventory layout");
            self.card_render.draw(*layout, card, game_time, framebuffer);
        }
    }
}
