use super::*;

// All relative
const COLUMN_SPACING: f32 = 0.04;
const ROW_SPACING: f32 = 0.125;
const BORDER_SPACING: f32 = 0.03;
const CARD_EXTRA_SPACE: f32 = 0.05;
const CLANS_WIDTH: f32 = 0.1;
const BUTTON_WIDTH: f32 = 0.15;
const BUTTON_SPACING: f32 = 0.03;
const GO_SIZE: f32 = 0.1;

/// Height divided by width
pub const CARD_SIZE_RATIO: f32 = 1.3269;

pub struct LayoutWidget {
    pub position: AABB<f32>,
    pub hovered: bool,
    pub pressed: bool,
}

impl Default for LayoutWidget {
    fn default() -> Self {
        Self {
            position: AABB::ZERO,
            hovered: false,
            pressed: false,
        }
    }
}

impl LayoutWidget {
    pub fn new(position: AABB<f32>) -> Self {
        Self {
            position,
            hovered: false,
            pressed: false,
        }
    }

    pub fn update(&mut self, position: AABB<f32>) {
        self.position = position;
    }
}

pub struct ShopLayout {
    pub tier_up: LayoutWidget,
    pub current_tier: LayoutWidget,
    pub currency: LayoutWidget,
    pub reroll: LayoutWidget,
    pub freeze: LayoutWidget,
    pub shop: LayoutWidget,
    pub shop_cards: Vec<LayoutWidget>,
    pub clans: LayoutWidget,
    pub go: LayoutWidget,
    pub inventory: LayoutWidget,
    pub inventory_cards: Vec<LayoutWidget>,
    pub drag_card_size: Vec2<f32>,
}

impl Default for ShopLayout {
    fn default() -> Self {
        Self {
            tier_up: default(),
            current_tier: default(),
            currency: default(),
            reroll: default(),
            freeze: default(),
            shop: default(),
            shop_cards: default(),
            go: default(),
            clans: default(),
            inventory: default(),
            inventory_cards: default(),
            drag_card_size: Vec2::ZERO,
        }
    }
}

impl ShopLayout {
    pub fn new(
        screen_size: Vec2<f32>,
        shop_cards: usize,
        party_cards: usize,
        inventory_cards: usize,
    ) -> Self {
        let mut shop = Self::default();
        shop.update(screen_size, shop_cards, party_cards, inventory_cards);
        shop
    }

    pub fn update(
        &mut self,
        screen_size: Vec2<f32>,
        shop_cards: usize,
        party_cards: usize,
        inventory_cards: usize,
    ) {
        let screen = AABB::point(screen_size * BORDER_SPACING);
        let screen = screen.extend_positive(screen_size * (1.0 - BORDER_SPACING * 2.0));

        let column_spacing = COLUMN_SPACING * screen.width();
        let row_spacing = ROW_SPACING * screen.height();

        let row_height = (screen.height() - row_spacing * 2.0) / 3.0;
        let card_extra_space = row_height * CARD_EXTRA_SPACE;
        let card_height = row_height - card_extra_space * 2.0;
        let card_width = card_height / CARD_SIZE_RATIO;
        let card_size = vec2(card_width, card_height);

        let button_spacing = BUTTON_SPACING * screen.height();
        let button_width = BUTTON_WIDTH * screen.width();
        let button_height = (row_height - button_spacing * 2.0) / 3.0;

        let bottom_row =
            AABB::point(screen.bottom_left()).extend_positive(vec2(screen.width(), row_height));
        let middle_row = bottom_row.translate(vec2(0.0, row_height + row_spacing));
        let top_row = middle_row.translate(vec2(0.0, row_height + row_spacing));

        let layout_cards_aabb = |max_space: AABB<f32>, count| {
            let mut card_size = card_size;
            let mut width = card_size.x * count as f32 + card_extra_space * (count + 1) as f32;
            let mut height = row_height;
            if width > max_space.width() {
                let scale = max_space.width() / width;
                width *= scale;
                height *= scale;
                card_size *= scale;
            }
            let aabb = AABB::point(max_space.center()).extend_symmetric(vec2(width, height) / 2.0);
            (aabb, card_size)
        };
        let layout_cards = |bottom_left, count, card_size: Vec2<f32>| {
            let card_extra_space = card_size.y * CARD_EXTRA_SPACE;
            (0..count)
                .map(|i| {
                    AABB::point(
                        bottom_left
                            + vec2(card_extra_space, card_extra_space)
                            + vec2((card_size.x + card_extra_space) * i as f32, 0.0),
                    )
                    .extend_positive(card_size)
                })
                .collect::<Vec<_>>()
        };

        // Inventory
        let (inventory, inventory_card) = layout_cards_aabb(bottom_row, inventory_cards);
        let inventory_cards =
            layout_cards(inventory.bottom_left(), inventory_cards, inventory_card);

        let clans_width = CLANS_WIDTH * screen.width();
        let go_size = GO_SIZE * screen.height();
        let mid_width = column_spacing + clans_width + column_spacing + go_size;

        // Shop
        let (shop, shop_card) =
            layout_cards_aabb(middle_row.extend_right(-mid_width), shop_cards);
        let mid_width = mid_width + shop.width();
        let bot_left = middle_row.center() - vec2(mid_width, shop.height()) / 2.0;
        let shop_cards = layout_cards(bot_left, shop_cards, shop_card);

        let mut bot_left = middle_row.center() - vec2(mid_width, row_height) / 2.0;
        bot_left.x += shop.width() + column_spacing;

        // Clans
        let clans = AABB::point(bot_left).extend_positive(vec2(clans_width, row_height));
        bot_left.x += clans_width + column_spacing;

        // Go button
        let go = AABB::point(screen.bottom_right()).extend_left(go_size).extend_up(go_size);

        // Top left buttons
        let top_left_buttons = AABB::point(screen.top_left())
            .extend_right(button_width)
            .extend_down(row_height);

        // Top right buttons
        let top_right_buttons = AABB::point(screen.top_right())
            .extend_left(button_width)
            .extend_down(row_height);

        // Tier up
        let tier_up = AABB::point(top_left_buttons.top_left())
            .extend_right(button_width)
            .extend_down(button_height);

        // Current tier
        let current_tier = tier_up.translate(vec2(0.0, -button_height - button_spacing));

        // Available currency
        let currency = current_tier.translate(vec2(0.0, -button_height - button_spacing));

        // Reroll button
        let reroll = AABB::point(top_right_buttons.top_left())
            .extend_right(button_width)
            .extend_down(button_height);

        // Freeze button
        let freeze = reroll.translate(vec2(0.0, -button_height - button_spacing));

        self.tier_up.update(tier_up);
        self.current_tier.update(current_tier);
        self.currency.update(currency);
        self.reroll.update(reroll);
        self.freeze.update(freeze);
        self.shop.update(shop);
        self.clans.update(clans);
        self.go.update(go);
        self.inventory.update(inventory);
        self.drag_card_size = card_size;
        vec_update(&mut self.shop_cards, &shop_cards);
        vec_update(&mut self.inventory_cards, &inventory_cards);
    }

    pub fn walk_widgets_mut(&mut self, f: &mut impl FnMut(&mut LayoutWidget)) {
        f(&mut self.tier_up);
        f(&mut self.current_tier);
        f(&mut self.currency);
        f(&mut self.reroll);
        f(&mut self.freeze);
        f(&mut self.shop);
        f(&mut self.clans);
        f(&mut self.go);
        f(&mut self.inventory);
        self.shop_cards
            .iter_mut()
            .chain(&mut self.inventory_cards)
            .for_each(|widget| f(widget));
    }
}

fn vec_update(vec: &mut Vec<LayoutWidget>, updates: &[AABB<f32>]) {
    let mut widgets = vec.iter_mut();
    let mut updates = updates.iter().copied();
    loop {
        let widget = widgets.next();
        let update = updates.next();
        match (widget, update) {
            (Some(widget), Some(update)) => {
                widget.update(update);
            }
            (Some(widget), None) => {
                let delta = vec.len() - updates.len();
                for _ in 0..delta {
                    vec.remove(vec.len() - 1);
                }
                break;
            }
            (None, Some(update)) => {
                vec.push(LayoutWidget::new(update));
                vec.extend(updates.map(|position| LayoutWidget::new(position)));
                break;
            }
            (None, None) => break,
        }
    }
}
