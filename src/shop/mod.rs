pub mod render;
mod unit_card;

use super::*;
use crate::render::UnitRender;

use geng::MouseButton;
use unit_card::*;

const UNIT_COST: Money = 3;
const MAX_PARTY: usize = 7;
const MAX_INVENTORY: usize = 10;

pub struct ShopState {
    geng: Geng,
    pub shop: Shop,
    pub render_shop: render::RenderShop,
    pub render: render::Render,
    pub time: Time,
}

impl ShopState {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: ShopConfig) -> Self {
        Self {
            geng: geng.clone(),
            shop: Shop::new(geng, assets, config.units.map.values().cloned()), // TODO: possibly optimize
            render_shop: render::RenderShop::new(vec2(1.0, 1.0), 0, 0, 0),
            render: render::Render::new(geng, assets, config),
            time: Time::ZERO,
        }
    }
}

impl geng::State for ShopState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.render_shop.layout.update(
            framebuffer.size().map(|x| x as _),
            self.shop.shop.len(),
            self.shop.party.len(),
            self.shop.inventory.len(),
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
                            Interaction::Reroll => self.shop.reroll(),
                            Interaction::Freeze => self.shop.freeze(),
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
}

pub enum Interaction {
    TierUp,
    Reroll,
    Freeze,
    Card(CardState),
}

pub type Money = u32;

pub struct Shop {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub tier: Tier,
    pub money: Money,
    pub frozen: bool,
    pub available: Vec<UnitTemplate>,
    pub shop: Vec<Option<UnitCard>>,
    pub party: Vec<Option<UnitCard>>,
    pub inventory: Vec<Option<UnitCard>>,
    pub drag: Option<Drag>,
}

pub struct Drag {
    pub start_position: Vec2<f32>,
    pub position: Vec2<f32>,
    pub target: DragTarget,
}

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

        None
    }

    pub fn drag_card(&mut self, state: CardState, position: Vec2<f32>) {
        self.drag_stop();
        let card = self.shop.get_card_mut(&state).and_then(|card| card.take());
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
                            Interaction::Card(state) => match self.shop.get_card_mut(&state) {
                                Some(target @ None) => {
                                    // Change placement
                                    *target = Some(card);
                                    return;
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }

                    // Return to old state, aka drop
                    match old_state {
                        CardState::Shop { index } => {
                            *self.shop.shop.get_mut(index).unwrap() = Some(card);
                        }
                        CardState::Party { index } => {
                            *self.shop.party.get_mut(index).unwrap() = Some(card);
                        }
                        CardState::Inventory { index } => {
                            *self.shop.inventory.get_mut(index).unwrap() = Some(card);
                        }
                    }
                }
            }
        }
    }
}

impl Shop {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        units: impl Iterator<Item = UnitTemplate>,
    ) -> Self {
        let mut shop = Self {
            geng: geng.clone(),
            assets: assets.clone(),
            tier: 1,
            money: 0,
            frozen: false,
            shop: vec![],
            party: vec![None; MAX_PARTY],
            inventory: vec![None; MAX_INVENTORY],
            drag: None,
            available: units
                .filter(|unit| unit.tier > 0)
                .map(|unit| unit)
                .collect(),
        };
        shop.reroll();
        shop
    }

    pub fn get_card_mut(&mut self, state: &CardState) -> Option<&mut Option<UnitCard>> {
        match state {
            &CardState::Shop { index } => self.shop.get_mut(index),
            &CardState::Party { index } => self.party.get_mut(index),
            &CardState::Inventory { index } => self.inventory.get_mut(index),
        }
    }

    pub fn tier_up(&mut self) {
        if let Some(cost) = tier_up_cost(self.tier) {
            if self.money >= cost {
                self.tier += 1;
                self.money -= cost;
            }
        }
    }

    pub fn reroll(&mut self) {
        if let Some(units) = tier_units_number(self.tier) {
            self.shop = self
                .available
                .iter()
                .filter(|unit| unit.tier <= self.tier)
                .map(|unit| Some(UnitCard::new(unit.clone())))
                .choose_multiple(&mut global_rng(), units);
        }
    }

    pub fn freeze(&mut self) {
        self.frozen = !self.frozen;
    }
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

const TIER_UP: [Money; 5] = [5, 6, 7, 8, 9];
const TIER_UNITS: [usize; 6] = [3, 4, 4, 5, 5, 6];

fn tier_up_cost(current_tier: Tier) -> Option<Money> {
    TIER_UP.get(current_tier as usize - 1).copied()
}

fn tier_units_number(current_tier: Tier) -> Option<usize> {
    TIER_UNITS.get(current_tier as usize - 1).copied()
}
