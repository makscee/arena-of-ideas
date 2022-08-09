use super::*;

// All relative
const CARD_HEIGHT: f32 = 0.2;
const BORDER_SPACING: f32 = 0.03;
const CARD_EXTRA_SPACE: f32 = 0.05;
const CLANS_WIDTH: f32 = 0.1;
const BUTTON_WIDTH: f32 = 0.15;
const BUTTON_SPACING: f32 = 0.03;
const GO_SIZE: f32 = 0.1;

const CURRENCY_BUTTON_WIDTH: f32 = 0.3;
const CURRENCY_BUTTON_HEIGHT: f32 = 0.1;

const CURRENT_TIER_WIDTH: f32 = 0.2;
const CURRENT_TIER_HEIGHT: f32 = 0.1;

const TIER_UP_BUTTON_WIDTH: f32 = 0.25;
const TIER_UP_BUTTON_HEIGHT: f32 = 0.075;

const REROLL_BUTTON_WIDTH: f32 = 0.25;
const REROLL_BUTTON_HEIGHT: f32 = 0.075;

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

        let card_height = CARD_HEIGHT * screen.height();
        let card_extra_space = card_height * CARD_EXTRA_SPACE;
        let card_width = card_height / CARD_SIZE_RATIO;
        let card_size = vec2(card_width, card_height);

        let button_spacing = BUTTON_SPACING * screen.height();

        let layout_cards_aabb = |max_space: AABB<f32>, count| {
            let mut card_size = card_size;
            let mut width = card_size.x * count as f32 + card_extra_space * (count + 1) as f32;
            let mut height = card_size.y + card_extra_space * 2.0;
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
        let inventory =
            AABB::point(screen.bottom_left()).extend_positive(vec2(screen.width(), card_height));
        let (inventory, inventory_card) = layout_cards_aabb(inventory, inventory_cards);
        let inventory_cards =
            layout_cards(inventory.bottom_left(), inventory_cards, inventory_card);

        // Shop
        let x_min = screen.center().x + card_extra_space;
        let shop = AABB::point(vec2(x_min, screen.center().y))
            .extend_right(screen.width() - x_min + screen.x_min)
            .extend_symmetric(vec2(0.0, card_height) / 2.0);
        let (shop, shop_card) = layout_cards_aabb(shop, shop_cards);
        let shop = shop.translate(vec2(x_min - shop.x_min, 0.0));
        let shop_cards = layout_cards(shop.bottom_left(), shop_cards, shop_card);

        // Available currency
        let currency = AABB::point(vec2(screen.center().x, screen.y_max))
            .extend_symmetric(vec2(CURRENCY_BUTTON_WIDTH * screen.height(), 0.0) / 2.0)
            .extend_down(CURRENCY_BUTTON_HEIGHT * screen.height());

        // Current tier
        let current_tier = AABB::point(vec2(
            currency.x_max + button_spacing,
            shop.y_max + button_spacing,
        ))
        .extend_right(CURRENT_TIER_WIDTH * screen.height())
        .extend_up(CURRENT_TIER_HEIGHT * screen.height());

        // Tier up
        let tier_up = AABB::point(vec2(
            current_tier.x_max + button_spacing,
            current_tier.center().y,
        ))
        .extend_right(TIER_UP_BUTTON_WIDTH * screen.height())
        .extend_symmetric(vec2(0.0, TIER_UP_BUTTON_HEIGHT * screen.height()) / 2.0);

        // Reroll button
        let reroll = AABB::point(vec2(
            (current_tier.x_max + tier_up.x_min) / 2.0,
            shop_cards
                .first()
                .map(|aabb| aabb.y_min)
                .unwrap_or(current_tier.y_min)
                - button_spacing,
        ))
        .extend_symmetric(vec2(REROLL_BUTTON_WIDTH * screen.height(), 0.0) / 2.0)
        .extend_down(REROLL_BUTTON_HEIGHT * screen.height());

        // Clans
        let clans_width = CLANS_WIDTH * screen.width();
        let clans = AABB::point(screen.top_left())
            .extend_right(clans_width)
            .extend_down(clans_width);

        // Go button
        let go_size = GO_SIZE * screen.height();
        let go = AABB::point(screen.bottom_right())
            .extend_left(go_size)
            .extend_up(go_size);

        self.tier_up.update(tier_up);
        self.current_tier.update(current_tier);
        self.currency.update(currency);
        self.reroll.update(reroll);
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
        f(&mut self.shop);
        f(&mut self.clans);
        f(&mut self.go);
        f(&mut self.inventory);
        self.shop_cards
            .iter_mut()
            .chain(&mut self.inventory_cards)
            .for_each(f);
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
                vec.extend(updates.map(LayoutWidget::new));
                break;
            }
            (None, None) => break,
        }
    }
}
