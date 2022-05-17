pub mod render;
mod unit_card;

use super::*;
use crate::render::UnitRender;

use geng::MouseButton;
use unit_card::*;

const MAX_PARTY: usize = 7;
const MAX_INVENTORY: usize = 10;
const UNIT_COST: Money = 3;
const UNIT_SELL_COST: Money = 1;
const REROLL_COST: Money = 1;
const TIER_UP_COST: [Money; 5] = [5, 6, 7, 8, 9];
const TIER_UNITS: [usize; 6] = [3, 4, 4, 5, 5, 6];

pub struct ShopState {
    geng: Geng,
    assets: Rc<Assets>,
    pub shop: Shop,
    game_config: Config,
    render_shop: render::RenderShop,
    render: render::Render,
    time: Time,
    transition: bool,
}

impl ShopState {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: ShopConfig, game_config: Config) -> Self {
        let shop = Shop::new(geng, assets, config);
        Self::load(geng, assets, shop, game_config)
    }

    pub fn load(geng: &Geng, assets: &Rc<Assets>, mut shop: Shop, game_config: Config) -> Self {
        shop.money = 10.min(4 + shop.round as Money);
        shop.round += 1;
        if !shop.frozen {
            shop.reroll(true);
        }
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            render_shop: render::RenderShop::new(vec2(1.0, 1.0), 0, 0, 0),
            render: render::Render::new(geng, assets),
            time: Time::ZERO,
            transition: false,
            game_config,
            shop,
        }
    }
}

impl geng::State for ShopState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.render_shop.layout.update(
            framebuffer.size().map(|x| x as _),
            self.shop.cards.shop.len(),
            self.shop.cards.party.len(),
            self.shop.cards.inventory.len(),
        );
        self.render.draw(
            &self.shop,
            &self.render_shop,
            self.time.as_f32(),
            framebuffer,
        );
    }

    fn update(&mut self, delta_time: f64) {
        self.time += Time::new(delta_time as _);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { position, button } => {
                if let MouseButton::Left = button {
                    let position = position.map(|x| x as f32);
                    if let Some((interaction, layout)) = self.get_under_pos_mut(position) {
                        layout.hovered = true;
                        layout.pressed = true;
                        match interaction {
                            Interaction::TierUp => self.shop.tier_up(),
                            Interaction::Reroll => self.shop.reroll(false),
                            Interaction::Freeze => self.shop.freeze(),
                            Interaction::Go => self.transition = true,
                            Interaction::Card(card) => {
                                self.drag_card(card, position);
                            }
                        }
                    }
                }
            }
            geng::Event::MouseUp { position, button } => {
                if let MouseButton::Left = button {
                    self.render_shop
                        .layout
                        .walk_widgets_mut(&mut |widget| widget.pressed = false);
                    self.drag_stop();
                }
            }
            geng::Event::MouseMove { position, .. } => {
                self.render_shop
                    .layout
                    .walk_widgets_mut(&mut |widget| widget.hovered = false);
                if let Some((_, layout)) =
                    self.get_under_pos_mut(self.geng.window().mouse_pos().map(|x| x as _))
                {
                    layout.hovered = true;
                }

                if let Some(drag) = &mut self.shop.drag {
                    drag.position = position.map(|x| x as _);
                }
            }
            _ => {}
        }
    }

    fn transition(&mut self) -> Option<geng::Transition> {
        if !self.transition {
            return None;
        }
        let config = Config {
            player: self
                .shop
                .cards
                .party
                .iter()
                .filter_map(|card| card.as_ref())
                .map(|card| card.unit.unit_type.clone())
                .collect(),
            alliances: {
                calc_alliances(
                    self.shop
                        .cards
                        .party
                        .iter()
                        .filter_map(|card| card.as_ref())
                        .map(|card| &card.template),
                )
            },
            ..self.game_config.clone()
        };
        let round = self
            .assets
            .rounds
            .get(self.shop.round - 1)
            .expect(&format!("Failed to find round number: {}", self.shop.round))
            .clone();
        let game_state = Game::new(&self.geng, &self.assets, config, self.shop.take(), round);
        Some(geng::Transition::Switch(Box::new(game_state)))
    }
}

pub enum Interaction {
    TierUp,
    Reroll,
    Freeze,
    Go,
    Card(CardState),
}

pub type Money = u32;

#[derive(Clone)]
pub struct Shop {
    pub config: ShopConfig,
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub round: usize,
    pub tier: Tier,
    pub money: Money,
    pub frozen: bool,
    pub available: Vec<(UnitType, UnitTemplate)>,
    pub cards: Cards,
    pub drag: Option<Drag>,
}

#[derive(Clone)]
pub struct Cards {
    pub shop: Vec<Option<UnitCard>>,
    pub party: Vec<Option<UnitCard>>,
    pub inventory: Vec<Option<UnitCard>>,
}

#[derive(Clone)]
pub struct Drag {
    pub start_position: Vec2<f32>,
    pub position: Vec2<f32>,
    pub target: DragTarget,
}

#[derive(Clone)]
pub enum DragTarget {
    Card {
        card: UnitCard,
        old_state: CardState,
    },
}

impl ShopState {
    pub fn get_under_pos_mut(
        &mut self,
        position: Vec2<f32>,
    ) -> Option<(Interaction, &mut render::LayoutWidget)> {
        let layout = &mut self.render_shop.layout;
        if let Some((index, layout)) = layout
            .shop_cards
            .iter_mut()
            .enumerate()
            .find(|(_, layout)| layout.position.contains(position))
        {
            return Some((Interaction::Card(CardState::Shop { index }), layout));
        }
        if let Some((index, layout)) = layout
            .party_cards
            .iter_mut()
            .enumerate()
            .find(|(_, layout)| layout.position.contains(position))
        {
            return Some((Interaction::Card(CardState::Party { index }), layout));
        }
        if let Some((index, layout)) = layout
            .inventory_cards
            .iter_mut()
            .enumerate()
            .find(|(_, layout)| layout.position.contains(position))
        {
            return Some((Interaction::Card(CardState::Inventory { index }), layout));
        }

        if layout.tier_up.position.contains(position) {
            return Some((Interaction::TierUp, &mut layout.tier_up));
        }
        if layout.reroll.position.contains(position) {
            return Some((Interaction::Reroll, &mut layout.reroll));
        }
        if layout.freeze.position.contains(position) {
            return Some((Interaction::Freeze, &mut layout.freeze));
        }
        if layout.go.position.contains(position) {
            return Some((Interaction::Go, &mut layout.go));
        }

        None
    }

    pub fn drag_card(&mut self, state: CardState, position: Vec2<f32>) {
        self.drag_stop();
        let card = self
            .shop
            .cards
            .get_card_mut(&state)
            .and_then(|card| card.take());
        if let Some(card) = card {
            self.shop.drag = Some(Drag {
                start_position: position,
                position,
                target: DragTarget::Card {
                    card,
                    old_state: state,
                },
            })
        }
    }

    pub fn drag_stop(&mut self) {
        if let Some(drag) = self.shop.drag.take() {
            match drag.target {
                DragTarget::Card { card, old_state } => {
                    if let Some((interaction, _)) = self.get_under_pos_mut(drag.position) {
                        match interaction {
                            Interaction::Card(state) => {
                                let from_shop = matches!(old_state, CardState::Shop { .. });
                                let to_shop = matches!(state, CardState::Shop { .. });
                                if !from_shop && to_shop {
                                    // Moved to shop -> sell
                                    self.shop.money += UNIT_SELL_COST;
                                    return;
                                }
                                match self.shop.cards.get_card_mut(&state) {
                                    Some(target @ None) => {
                                        if from_shop && !to_shop {
                                            // Moved from the shop -> check payment
                                            if self.shop.money >= UNIT_COST {
                                                self.shop.money -= UNIT_COST;
                                                *target = Some(card);
                                                return;
                                            }
                                        } else {
                                            // Change placement
                                            *target = Some(card);
                                            return;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }

                    // Return to old state, aka drop
                    match old_state {
                        CardState::Shop { index } => {
                            *self.shop.cards.shop.get_mut(index).unwrap() = Some(card);
                        }
                        CardState::Party { index } => {
                            *self.shop.cards.party.get_mut(index).unwrap() = Some(card);
                        }
                        CardState::Inventory { index } => {
                            *self.shop.cards.inventory.get_mut(index).unwrap() = Some(card);
                        }
                    }
                }
            }
        }
    }
}

impl Shop {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: ShopConfig) -> Self {
        let units = config
            .units
            .iter()
            .map(|unit_type| {
                let unit = assets
                    .units
                    .get(unit_type)
                    .expect(&format!("Failed to find unit: {unit_type}"));
                (unit_type, unit)
            })
            .filter(|(_, unit)| unit.tier > 0)
            .map(|(name, unit)| (name.clone(), unit.clone()))
            .collect();
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            round: 0,
            tier: 1,
            money: 0,
            frozen: false,
            cards: Cards::new(),
            drag: None,
            available: units,
            config,
        }
    }

    pub fn take(&mut self) -> Self {
        let mut shop = Shop::new(&self.geng, &self.assets, default());
        std::mem::swap(self, &mut shop);
        shop
    }

    pub fn tier_up(&mut self) {
        if let Some(cost) = tier_up_cost(self.tier) {
            if self.money >= cost {
                self.tier += 1;
                self.money -= cost;
            }
        }
    }

    /// Rerolls the shop units. If `force` is true, then the cost is not paid.
    pub fn reroll(&mut self, force: bool) {
        if self.money >= REROLL_COST || force {
            if !force {
                self.money -= REROLL_COST;
            }
            if let Some(units) = tier_units_number(self.tier) {
                self.cards.shop = self
                    .available
                    .iter()
                    .filter(|(_, unit)| unit.tier <= self.tier)
                    .map(|(unit_type, unit)| Some(UnitCard::new(unit.clone(), unit_type.clone())))
                    .choose_multiple(&mut global_rng(), units);
            }
        }
    }

    pub fn freeze(&mut self) {
        self.frozen = !self.frozen;
    }
}

impl Cards {
    pub fn new() -> Self {
        Self {
            shop: vec![],
            party: vec![None; MAX_PARTY],
            inventory: vec![None; MAX_INVENTORY],
        }
    }

    pub fn get_card_mut(&mut self, state: &CardState) -> Option<&mut Option<UnitCard>> {
        match state {
            &CardState::Shop { index } => self.shop.get_mut(index),
            &CardState::Party { index } => self.party.get_mut(index),
            &CardState::Inventory { index } => self.inventory.get_mut(index),
        }
    }
}

fn calc_alliances<'a>(
    units: impl IntoIterator<Item = &'a UnitTemplate>,
) -> HashMap<Alliance, usize> {
    let mut alliances = HashMap::new();
    for template in units {
        for alliance in &template.alliances {
            *alliances.entry(alliance.clone()).or_insert(0) += 1;
        }
    }
    alliances
}

fn roll(choices: &[UnitTemplate], tier: Tier, units: usize) -> Vec<UnitTemplate> {
    choices
        .iter()
        .filter(|unit| unit.tier <= tier)
        .map(|unit| unit.clone()) // TODO: optimize
        .choose_multiple(&mut global_rng(), units)
}

fn earn_money(round: usize) -> Money {
    (4 + round).min(10) as _
}

fn tier_up_cost(current_tier: Tier) -> Option<Money> {
    TIER_UP_COST.get(current_tier as usize - 1).copied()
}

fn tier_units_number(current_tier: Tier) -> Option<usize> {
    TIER_UNITS.get(current_tier as usize - 1).copied()
}
