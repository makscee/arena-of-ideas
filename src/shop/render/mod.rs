use super::*;
use geng::{prelude::itertools::Itertools, Draw2d};

mod card;
mod layout;

pub use card::*;
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
const BUTTON_HOVER_COLOR: Color<f32> = Color {
    r: 0.0,
    g: 0.5,
    b: 0.8,
    a: 1.0,
};
const BUTTON_PRESS_COLOR: Color<f32> = Color {
    r: 0.0,
    g: 0.3,
    b: 0.6,
    a: 1.0,
};
const TEXT_COLOR: Color<f32> = Color::WHITE;
const CLAN_MAX_SIZE: f32 = 0.15;
const BAR_SIZE: f32 = 0.1;
/// Relative to the framebuffer size
const CLAN_INFO_SIZE: Vec2<f32> = vec2(0.3, 0.2);
/// Relative to the clan info size
const FONT_SIZE: f32 = 0.1;
const CLAN_BACKGROUND_COLOR: Color<f32> = Color {
    r: 0.3,
    g: 0.3,
    b: 0.3,
    a: 1.0,
};
const CLAN_INFO_BACKGROUND_COLOR: Color<f32> = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

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
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
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
        let layout = &render.layout;

        draw_2d::Quad::new(layout.shop.position, TEXT_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );
        let mut selected_clan = None;
        for (index, card) in shop.cards.shop.iter().enumerate() {
            let layout = render
                .layout
                .shop_cards
                .get(index)
                .expect("Invalid shop layout");
            selected_clan = selected_clan.or(self.card_render.draw(
                layout.position,
                card.as_ref(),
                game_time,
                framebuffer,
            ));
        }

        draw_2d::Quad::new(layout.party.position, TEXT_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );
        for (index, card) in shop.cards.party.iter().enumerate() {
            let layout = render
                .layout
                .party_cards
                .get(index)
                .expect("Invalid party layout");
            selected_clan = selected_clan.or(self.card_render.draw(
                layout.position,
                card.as_ref(),
                game_time,
                framebuffer,
            ));
        }

        draw_2d::Quad::new(layout.inventory.position, TEXT_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );
        for (index, card) in shop.cards.inventory.iter().enumerate() {
            let layout = render
                .layout
                .inventory_cards
                .get(index)
                .expect("Invalid inventory layout");
            selected_clan = selected_clan.or(self.card_render.draw(
                layout.position,
                card.as_ref(),
                game_time,
                framebuffer,
            ));
        }

        let text = match tier_up_cost(shop.tier, shop.tier_rounds) {
            Some(cost) => format!("Tier Up ({})", cost),
            None => format!("Tier Up (?)"),
        };
        draw_rectangle(
            &text,
            layout.tier_up.position,
            button_color(&layout.tier_up),
            &self.geng,
            framebuffer,
        );

        draw_rectangle(
            &format!("Tier {}", shop.tier),
            layout.current_tier.position,
            TEXT_BACKGROUND_COLOR,
            &self.geng,
            framebuffer,
        );

        let text = if shop.money == 1 { "coin" } else { "coins" };
        draw_rectangle(
            &format!("{} {}", shop.money, text),
            layout.currency.position,
            TEXT_BACKGROUND_COLOR,
            &self.geng,
            framebuffer,
        );

        draw_rectangle(
            &format!("Reroll"),
            layout.reroll.position,
            button_color(&layout.reroll),
            &self.geng,
            framebuffer,
        );

        draw_rectangle(
            &format!("Freeze"),
            layout.freeze.position,
            button_color(&layout.freeze),
            &self.geng,
            framebuffer,
        );

        draw_rectangle(
            "",
            layout.clans.position,
            TEXT_BACKGROUND_COLOR,
            &self.geng,
            framebuffer,
        );

        let clans = calc_clan_members(
            shop.cards
                .party
                .iter()
                .filter_map(|card| card.as_ref())
                .map(|card| &card.unit),
        );
        let clans = clans.into_iter().sorted().collect::<Vec<_>>();
        if !clans.is_empty() {
            let height = layout.clans.position.height();
            let size = (CLAN_MAX_SIZE * height).min(height / clans.len() as f32);
            let clan_size = vec2(size, size);
            let mut position = layout.clans.position.top_left() + vec2(size, -size) / 2.0;
            for (clan, clan_count) in clans {
                let clan_color = self
                    .assets
                    .options
                    .clan_colors
                    .get(&clan)
                    .copied()
                    .unwrap_or(Color::WHITE);
                let text_color = Color::WHITE;
                let text = format!("{:?}", clan)
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                draw_2d::Ellipse::circle(position, size / 2.0, clan_color).draw_2d(
                    &self.geng,
                    framebuffer,
                    camera,
                );
                draw_2d::Text::unit(&**self.geng.default_font(), text, text_color)
                    .fit_into(AABB::point(position).extend_uniform(size / 2.0 / 2.0.sqrt()))
                    .draw_2d(&self.geng, framebuffer, camera);

                if let Some(config) = shop.config.render.clans.get(&clan) {
                    let bar_size = vec2(size * BAR_SIZE, size / config.rows as f32);
                    for x in 0..config.columns {
                        for y in 0..config.rows {
                            let position = position
                                + clan_size / 2.0
                                + bar_size * vec2(x as f32, -(y as f32) - 1.0);
                            let color = if x * config.rows + y + 1 <= clan_count {
                                clan_color
                            } else {
                                CLAN_BACKGROUND_COLOR
                            };
                            draw_2d::Quad::new(
                                AABB::point(position).extend_positive(bar_size),
                                color,
                            )
                            .draw_2d(&self.geng, framebuffer, camera);
                        }
                    }

                    let mouse_pos = self.geng.window().mouse_pos().map(|x| x as f32);
                    if AABB::point(position)
                        .extend_uniform(size / 2.0)
                        .contains(mouse_pos)
                    {
                        selected_clan = selected_clan.or(Some(clan));
                    }
                }

                position.y -= size;
            }
        }

        draw_rectangle(
            &format!("Go"),
            layout.go.position,
            button_color(&layout.go),
            &self.geng,
            framebuffer,
        );

        if let Some(drag) = &shop.drag {
            match &drag.target {
                DragTarget::Card { card, .. } => {
                    let aabb =
                        AABB::point(drag.position).extend_symmetric(layout.drag_card_size / 2.0);
                    selected_clan = selected_clan.or(self.card_render.draw(
                        aabb,
                        Some(card),
                        game_time,
                        framebuffer,
                    ));
                }
            }
        }

        if let Some(clan) = selected_clan {
            if let Some(config) = shop.config.render.clans.get(&clan) {
                // Show clan info
                let mouse_pos = self.geng.window().mouse_pos().map(|x| x as f32);
                draw_clan_info(
                    mouse_pos,
                    &config.description,
                    &self.geng,
                    framebuffer,
                    camera,
                );
            }
        }
    }
}

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
    draw_text_wrapped(
        &**geng.default_font(),
        description,
        FONT_SIZE * info_aabb.height(),
        info_aabb,
        TEXT_COLOR,
        geng,
        framebuffer,
        camera,
    );
}

fn button_color(widget: &LayoutWidget) -> Color<f32> {
    if widget.pressed {
        BUTTON_PRESS_COLOR
    } else if widget.hovered {
        BUTTON_HOVER_COLOR
    } else {
        BUTTON_COLOR
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

fn draw_text_wrapped(
    font: impl std::borrow::Borrow<geng::Font>,
    text: impl AsRef<str>,
    font_size: f32,
    target: AABB<f32>,
    color: Color<f32>,
    geng: &Geng,
    framebuffer: &mut ugli::Framebuffer,
    camera: &impl geng::AbstractCamera2d,
) -> Option<()> {
    let max_width = target.width();
    let font = font.borrow();
    let text = text.as_ref();

    let mut pos = vec2(target.center().x, target.y_max - font_size);
    let measure = |text| font.measure_at(text, Vec2::ZERO, font_size);

    for line in text.lines() {
        let mut words = line.split_whitespace();
        let mut line = String::new();
        let mut line_width = 0.0;
        if let Some(word) = words.next() {
            let width = measure(word)?.width();
            line_width += width;
            line += word;
        }
        while let Some(word) = words.next() {
            let width = measure(word)?.width();
            if line_width + width <= max_width {
                line_width += width;
                line += " ";
                line += word;
            } else {
                font.draw(
                    framebuffer,
                    camera,
                    &line,
                    pos,
                    geng::TextAlign::CENTER,
                    font_size,
                    color,
                );
                pos.y -= font_size;
                line = String::new();
                line_width = width;
                line += word;
                continue;
            }
        }
        font.draw(
            framebuffer,
            camera,
            &line,
            pos,
            geng::TextAlign::CENTER,
            font_size,
            color,
        );
        pos.y -= font_size;
    }
    Some(())
}
