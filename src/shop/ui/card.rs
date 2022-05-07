use super::*;

/// Height divided by width
pub const CARD_SIZE_RATIO: f32 = 1.3269;

pub struct UnitCardWidget<'a> {
    pub render: UnitRender,
    pub sense: &'a mut Sense,
    pub card: Option<&'a UnitCard>,
    pub game_time: f32,
}

impl<'a> UnitCardWidget<'a> {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        cx: &'a Controller,
        card: Option<&'a UnitCard>,
        game_time: f32,
    ) -> Self {
        Self {
            sense: cx.get_state(),
            render: UnitRender::new(geng, assets),
            card,
            game_time,
        }
    }
}

impl<'a> Widget for UnitCardWidget<'a> {
    fn calc_constraints(&mut self, cx: &ConstraintsContext) -> Constraints {
        Constraints::default()
    }

    fn sense(&mut self) -> Option<&mut Sense> {
        Some(self.sense)
    }

    fn update(&mut self, delta_time: f64) {}

    fn draw(&mut self, cx: &mut DrawContext) {
        if let Some(card) = &self.card {
            let pixel_camera = &geng::PixelPerfectCamera;

            // Relative to the height
            // TODO: de-hardcode
            const TOP_SPACE: f32 = 0.1;
            const HERO_HEIGHT: f32 = 0.35;
            const HP_AABB: AABB<f32> = AABB {
                x_min: 0.02,
                x_max: 0.09,
                y_min: 0.02,
                y_max: 0.09,
            };
            const DAMAGE_AABB: AABB<f32> = AABB {
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

            // Card layout
            let card_aabb = cx.position.map(|x| x as f32);
            let height = card_aabb.height().min(card_aabb.width() * CARD_SIZE_RATIO);
            let width = height / CARD_SIZE_RATIO;
            let card_aabb =
                AABB::point(card_aabb.center()).extend_symmetric(vec2(width, height) / 2.0);

            // Hero layout
            let mut hero_aabb = card_aabb.extend_up(-TOP_SPACE * height);
            hero_aabb.y_min = hero_aabb.y_max - HERO_HEIGHT * height;
            let hero_aabb = hero_aabb;

            let mut temp_texture = ugli::Texture::new_with(
                cx.geng.ugli(),
                hero_aabb.size().map(|x| x.ceil() as _),
                |_| Color::TRANSPARENT_BLACK,
            );
            let mut temp_framebuffer = ugli::Framebuffer::new_color(
                cx.geng.ugli(),
                ugli::ColorAttachment::Texture(&mut temp_texture),
            );

            self.render.draw_unit(
                &card.unit,
                &card.template,
                None,
                self.game_time,
                &geng::Camera2d {
                    center: Vec2::ZERO,
                    rotation: 0.0,
                    fov: card.unit.radius.as_f32() * 1.5,
                },
                &mut temp_framebuffer,
            );

            // HP
            let hp_aabb = HP_AABB
                .map(|x| x * height)
                .translate(card_aabb.bottom_left());

            // Damage
            let damage_aabb = DAMAGE_AABB
                .map(|x| x * height)
                .translate(card_aabb.bottom_left());

            // Tier
            let tier_aabb = TIER_AABB
                .map(|x| x * height)
                .translate(card_aabb.bottom_left());

            // Render
            // Hero
            draw_2d::TexturedQuad::new(hero_aabb, &temp_texture).draw_2d(
                cx.geng,
                cx.framebuffer,
                pixel_camera,
            );

            // Card texture
            draw_2d::TexturedQuad::new(card_aabb, &*self.render.assets.card).draw_2d(
                cx.geng,
                cx.framebuffer,
                pixel_camera,
            );

            // HP
            draw_2d::Text::unit(
                &**cx.geng.default_font(),
                format!("{}", card.unit.health),
                Color::WHITE,
            )
            .fit_into(hp_aabb)
            .draw_2d(cx.geng, cx.framebuffer, pixel_camera);

            // Damage
            draw_2d::Text::unit(&**cx.geng.default_font(), format!("?"), Color::WHITE)
                .fit_into(damage_aabb)
                .draw_2d(cx.geng, cx.framebuffer, pixel_camera);

            // Tier
            draw_2d::Text::unit(
                &**cx.geng.default_font(),
                format!("Tier {}", card.template.tier),
                Color::WHITE,
            )
            .fit_into(tier_aabb)
            .draw_2d(cx.geng, cx.framebuffer, pixel_camera);
        }
    }

    fn handle_event(&mut self, event: &geng::Event) {}
}
