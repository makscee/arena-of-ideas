use super::*;

// All relative
const COLUMN_SPACING: f32 = 0.06;
const ROW_SPACING: f32 = 0.125;
const BORDER_SPACING: f32 = 0.03;
const CARD_EXTRA_SPACE: f32 = 0.05;
const ALLIANCES_WIDTH: f32 = 0.1;
const BUTTON_WIDTH: f32 = 0.15;
const BUTTON_SPACING: f32 = 0.03;

/// Height divided by width
pub const CARD_SIZE_RATIO: f32 = 1.3269;

pub struct ShopLayout {
    pub tier_up: AABB<f32>,
    pub current_tier: AABB<f32>,
    pub currency: AABB<f32>,
    pub reroll: AABB<f32>,
    pub freeze: AABB<f32>,
    pub shop: AABB<f32>,
    pub shop_cards: Vec<AABB<f32>>,
    pub party: AABB<f32>,
    pub party_cards: Vec<AABB<f32>>,
    pub alliances: AABB<f32>,
    pub inventory: AABB<f32>,
    pub inventory_cards: Vec<AABB<f32>>,
    pub drag_card_size: Vec2<f32>,
}

impl ShopLayout {
    pub fn new(
        screen_size: Vec2<f32>,
        shop_cards: usize,
        party_cards: usize,
        inventory_cards: usize,
    ) -> Self {
        let screen = AABB::point(screen_size * BORDER_SPACING);
        let screen = screen.extend_positive(screen_size * (1.0 - BORDER_SPACING * 2.0));

        let column_spacing = COLUMN_SPACING * screen.width();
        let row_spacing = ROW_SPACING * screen.height();

        let row_height = (screen.height() - row_spacing * 2.0) / 3.0;
        let card_extra_space = row_height * CARD_EXTRA_SPACE;
        let card_height = row_height - card_extra_space * 2.0;
        let card_width = card_height / CARD_SIZE_RATIO;
        let card_size = vec2(card_width, card_height);

        let bottom_row =
            AABB::point(screen.bottom_left()).extend_positive(vec2(screen.width(), row_height));
        let middle_row = bottom_row.translate(vec2(0.0, row_height + row_spacing));
        let top_row = middle_row.translate(vec2(0.0, row_height + row_spacing));

        let layout_cards_aabb = |max_space: AABB<f32>, count| {
            let width = card_size.x * count as f32 + card_extra_space * (count + 1) as f32;
            AABB::point(max_space.center()).extend_symmetric(vec2(width, row_height) / 2.0)
        };
        let layout_cards = |bottom_left, count| {
            (0..count)
                .map(|i| {
                    AABB::point(
                        bottom_left
                            + vec2(card_extra_space, card_extra_space)
                            + vec2((card_size.x + card_extra_space) * i as f32, 0.0),
                    )
                    .extend_positive(card_size)
                })
                .collect()
        };

        let inventory = layout_cards_aabb(bottom_row, inventory_cards);
        let inventory_cards = layout_cards(inventory.bottom_left(), inventory_cards);

        let alliances_width = ALLIANCES_WIDTH * screen.width();
        let party = layout_cards_aabb(
            middle_row.extend_right(-alliances_width - column_spacing),
            party_cards,
        );
        let mid_width = alliances_width + party.width() + column_spacing;
        let mut bot_left = middle_row.center() - vec2(mid_width, row_height) / 2.0;
        let party_cards = layout_cards(bot_left, party_cards);
        bot_left.x += party.width() + column_spacing;
        let alliances = AABB::point(bot_left).extend_positive(vec2(alliances_width, row_height));

        let button_spacing = BUTTON_SPACING * screen.height();
        let button_width = BUTTON_WIDTH * screen.width();
        let button_height = (row_height - button_spacing * 2.0) / 3.0;

        let top_left_buttons = AABB::point(screen.top_left())
            .extend_right(button_width)
            .extend_down(row_height);
        let top_right_buttons = AABB::point(screen.top_right())
            .extend_left(button_width)
            .extend_down(row_height);
        let shop = top_row
            .extend_right(-top_right_buttons.width() - column_spacing)
            .extend_left(-top_left_buttons.width() - column_spacing);
        let shop_cards = layout_cards(
            layout_cards_aabb(shop, shop_cards).bottom_left(),
            shop_cards,
        );

        let tier_up = AABB::point(top_left_buttons.top_left())
            .extend_right(button_width)
            .extend_down(button_height);
        let current_tier = tier_up.translate(vec2(0.0, -button_height - button_spacing));
        let currency = current_tier.translate(vec2(0.0, -button_height - button_spacing));

        let reroll = AABB::point(top_right_buttons.top_left())
            .extend_right(button_width)
            .extend_down(button_height);
        let freeze = reroll.translate(vec2(0.0, -button_height - button_spacing));

        Self {
            tier_up,
            current_tier,
            currency,
            reroll,
            freeze,
            shop,
            shop_cards,
            party,
            party_cards,
            alliances,
            inventory,
            inventory_cards,
            drag_card_size: card_size,
        }
    }
}
