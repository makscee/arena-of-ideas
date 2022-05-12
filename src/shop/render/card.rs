use geng::Draw2d;

use super::*;

// Relative to the height
// TODO: de-hardcode
const TOP_SPACE: f32 = 0.1;
const HERO_HEIGHT: f32 = 0.35;
const DAMAGE_AABB: AABB<f32> = AABB {
    x_min: 0.02,
    x_max: 0.09,
    y_min: 0.02,
    y_max: 0.09,
};
const HEALTH_AABB: AABB<f32> = AABB {
    x_min: 1.0 / CARD_SIZE_RATIO - 0.09,
    x_max: 1.0 / CARD_SIZE_RATIO - 0.02,
    y_min: 0.02,
    y_max: 0.09,
};
const TIER_AABB: AABB<f32> = AABB {
    x_min: 0.07,
    x_max: 0.25,
    y_min: 0.92,
    y_max: 0.97,
};
const ALLIANCE_AABB: AABB<f32> = AABB {
    x_min: 1.0 / CARD_SIZE_RATIO - 0.27,
    x_max: 1.0 / CARD_SIZE_RATIO - 0.05,
    y_min: 0.917,
    y_max: 0.973,
};
const NAME_AABB: AABB<f32> = AABB {
    x_min: 0.21,
    x_max: 0.53,
    y_min: 0.52,
    y_max: 0.58,
};
const DESCRIPTION_AABB: AABB<f32> = AABB {
    x_min: 0.04,
    x_max: 1.0 / CARD_SIZE_RATIO - 0.04,
    y_min: 0.17,
    y_max: 0.47,
};
const CARD_BACKGROUND_COLOR: Color<f32> = Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

pub struct CardRender {
    geng: Geng,
    render: UnitRender,
}

impl CardRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            render: UnitRender::new(geng, assets),
        }
    }

    pub fn draw(
        &mut self,
        card_aabb: AABB<f32>,
        card: Option<&UnitCard>,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let camera = &geng::PixelPerfectCamera;
        let width = card_aabb.width();
        let height = card_aabb.height();

        draw_2d::Quad::new(card_aabb, CARD_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );

        let card = match card {
            Some(card) => card,
            None => return,
        };

        // Hero layout
        let mut hero_aabb = card_aabb.extend_up(-TOP_SPACE * height);
        hero_aabb.y_min = hero_aabb.y_max - HERO_HEIGHT * height;
        let hero_aabb = hero_aabb;
        let mut temp_texture = ugli::Texture::new_with(
            self.geng.ugli(),
            hero_aabb.size().map(|x| x.ceil() as _),
            |_| Color::TRANSPARENT_BLACK,
        );
        let mut temp_framebuffer = ugli::Framebuffer::new_color(
            self.geng.ugli(),
            ugli::ColorAttachment::Texture(&mut temp_texture),
        );
        self.render.draw_unit(
            &card.unit,
            &card.template,
            None,
            game_time,
            &geng::Camera2d {
                center: Vec2::ZERO,
                rotation: 0.0,
                fov: card.unit.radius.as_f32() * 1.5,
            },
            &mut temp_framebuffer,
        );

        let layout = |aabb: AABB<f32>| aabb.map(|x| x * height).translate(card_aabb.bottom_left());
        let damage_aabb = layout(DAMAGE_AABB);
        let health_aabb = layout(HEALTH_AABB);
        let tier_aabb = layout(TIER_AABB);
        let alliance_aabb = layout(ALLIANCE_AABB);
        let name_aabb = layout(NAME_AABB);
        let description_aabb = layout(DESCRIPTION_AABB);

        // Render
        // Hero
        draw_2d::TexturedQuad::new(hero_aabb, &temp_texture).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );

        // Card texture
        draw_2d::TexturedQuad::new(card_aabb, &*self.render.assets.card).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );

        // Damage
        draw_2d::Text::unit(&**self.geng.default_font(), format!("?"), Color::WHITE)
            .fit_into(damage_aabb)
            .draw_2d(&self.geng, framebuffer, camera);

        // Health
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("{}", card.unit.health),
            Color::WHITE,
        )
        .fit_into(health_aabb)
        .draw_2d(&self.geng, framebuffer, camera);

        // Tier
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("Tier {}", card.template.tier),
            Color::WHITE,
        )
        .fit_into(tier_aabb)
        .draw_2d(&self.geng, framebuffer, camera);

        // Tier
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("TODO: Alliances"),
            Color::WHITE,
        )
        .fit_into(alliance_aabb)
        .draw_2d(&self.geng, framebuffer, camera);

        // Name
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("{}", card.unit.unit_type),
            Color::WHITE,
        )
        .fit_into(name_aabb)
        .draw_2d(&self.geng, framebuffer, camera);

        // Description
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("TODO: Description"),
            Color::WHITE,
        )
        .fit_into(description_aabb)
        .draw_2d(&self.geng, framebuffer, camera);
    }
}
