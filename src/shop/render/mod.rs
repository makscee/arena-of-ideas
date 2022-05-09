use super::*;

mod card;
mod layout;

pub use card::*;
use geng::Draw2d;
pub use layout::*;

const TEXT_OCCUPY_SPACE: f32 = 0.6;
const BACKGROUND_COLOR: Color<f32> = Color::BLACK;
const TEXT_BACKGROUND_COLOR: Color<f32> = Color {
    r: 0.2,
    g: 0.2,
    b: 0.2,
    a: 1.0,
};
const BUTTON_COLOR: Color<f32> = Color {
    r: 0.0,
    g: 0.7,
    b: 1.0,
    a: 1.0,
};
const TEXT_COLOR: Color<f32> = Color::WHITE;

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
        ugli::clear(framebuffer, Some(BACKGROUND_COLOR), None);
        let camera = &geng::PixelPerfectCamera;

        draw_2d::Quad::new(render.layout.shop, TEXT_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );
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

        draw_2d::Quad::new(render.layout.party, TEXT_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );
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

        draw_2d::Quad::new(render.layout.inventory, TEXT_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );
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

        let text = match tier_up_cost(shop.tier) {
            Some(cost) => format!("Tier Up ({})", cost),
            None => format!("Tier Up (?)"),
        };
        draw_rectangle(
            &text,
            render.layout.tier_up,
            BUTTON_COLOR,
            &self.geng,
            framebuffer,
        );

        draw_rectangle(
            &format!("Tier {}", shop.tier),
            render.layout.current_tier,
            TEXT_BACKGROUND_COLOR,
            &self.geng,
            framebuffer,
        );

        let text = if shop.money == 1 { "coin" } else { "coins" };
        draw_rectangle(
            &format!("{} {}", shop.money, text),
            render.layout.currency,
            TEXT_BACKGROUND_COLOR,
            &self.geng,
            framebuffer,
        );

        draw_rectangle(
            &format!("Reroll"),
            render.layout.reroll,
            BUTTON_COLOR,
            &self.geng,
            framebuffer,
        );

        draw_rectangle(
            &format!("Freeze"),
            render.layout.freeze,
            BUTTON_COLOR,
            &self.geng,
            framebuffer,
        );

        draw_2d::Quad::new(render.layout.alliances, TEXT_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );
    }
}

fn draw_rectangle(
    text: impl AsRef<str>,
    aabb: AABB<f32>,
    color: Color<f32>,
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
