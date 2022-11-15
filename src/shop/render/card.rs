use geng::Draw2d;
use strfmt::strfmt;

use super::*;

// Relative to the height
// TODO: de-hardcode
const TOP_SPACE: f32 = 0.1;
const FONT_SIZE: f32 = 0.08;
const HERO_HEIGHT: f32 = 0.35;
/// Height divided by width
const CARD_SIZE_RATIO: f32 = 1.3269;
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
const CLAN_AABB: AABB<f32> = AABB {
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
const CARD_BACKGROUND_COLOR: Rgba<f32> = Rgba {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};

pub struct CardRender {
    geng: Geng,
    assets: Rc<Assets>,
}

impl CardRender {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
        }
    }

    pub fn draw(
        &self,
        card_aabb: AABB<f32>,
        template: UnitTemplate,
        framebuffer: &mut ugli::Framebuffer,
        camera: &Camera2d,
        vars: HashMap<VarName, i32>,
    ) {
        let width = card_aabb.width();
        let height = card_aabb.height();

        draw_2d::Quad::new(card_aabb, CARD_BACKGROUND_COLOR).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );

        let layout = |aabb: AABB<f32>| aabb.map(|x| x * height).translate(card_aabb.bottom_left());
        let damage_aabb = layout(DAMAGE_AABB);
        let health_aabb = layout(HEALTH_AABB);
        let tier_aabb = layout(TIER_AABB);
        let clan_aabb = layout(CLAN_AABB);
        let name_aabb = layout(NAME_AABB);
        let description_aabb = layout(DESCRIPTION_AABB);

        // Card texture
        draw_2d::TexturedQuad::new(card_aabb, &*self.assets.card).draw_2d(
            &self.geng,
            framebuffer,
            camera,
        );

        // Damage
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("{}", template.attack),
            Rgba::WHITE,
        )
        .fit_into(damage_aabb)
        .draw_2d(&self.geng, framebuffer, camera);

        // Health
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("{}", template.health),
            Rgba::WHITE,
        )
        .fit_into(health_aabb)
        .draw_2d(&self.geng, framebuffer, camera);

        // Tier
        if template.tier > 0 {
            draw_2d::Text::unit(
                &**self.geng.default_font(),
                format!("Tier {}", template.tier),
                Rgba::WHITE,
            )
            .fit_into(tier_aabb)
            .draw_2d(&self.geng, framebuffer, camera);
        }

        // Clans
        let mut selected_clan = None;
        {
            let clans = template.clans.iter().sorted().collect::<Vec<_>>();
            let size = clan_aabb.height();
            let clan_size = vec2(size, size);
            let mut position = clan_aabb.top_left() + vec2(size, -size) / 2.0;
            for clan in clans {
                let clan_config = self.assets.options.clan_configs.get(&clan);
                let mut clan_color = Rgba::BLACK;
                if let Some(clan_config) = clan_config {
                    clan_color = clan_config.color;
                }

                let text_color = Rgba::WHITE;
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
                let mouse_pos = self.geng.window().mouse_pos().map(|x| x as f32);
                if AABB::point(position)
                    .extend_uniform(size / 2.0)
                    .contains(mouse_pos)
                {
                    selected_clan = Some(*clan);
                }
                position.x += size;
            }
        }

        // Name
        draw_2d::Text::unit(
            &**self.geng.default_font(),
            format!("{}", template.name),
            Rgba::WHITE,
        )
        .fit_into(name_aabb)
        .draw_2d(&self.geng, framebuffer, camera);

        // Description
        let font_size = FONT_SIZE * height;
        let text = strfmt(&template.description, &vars).unwrap_or(template.description);
        crate::render::draw_text_wrapped(
            &**self.geng.default_font(),
            &text,
            font_size,
            description_aabb,
            Rgba::WHITE,
            framebuffer,
            camera,
        );
    }
}
