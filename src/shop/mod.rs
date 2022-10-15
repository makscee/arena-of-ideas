pub mod render;
mod unit_card;

use super::*;

use geng::MouseButton;
use unit_card::*;

const MAX_PARTY: usize = 6;
const MAX_INVENTORY: usize = 7;
const UNIT_COST: Money = 3;
const UNIT_SELL_COST: Money = 1;
const REROLL_COST: Money = 1;
const TIER_UP_COST: [Money; 5] = [5, 7, 8, 9, 10];
const TIER_UNITS: [usize; 6] = [3, 4, 4, 5, 5, 6];

pub struct ShopState {
    geng: Geng,
    assets: Rc<Assets>,
    pub shop: Shop,
    game_config: Config,
    render_shop: render::RenderShop,
    render: render::Render,
    time: f64,
    transition: bool,
}

impl ShopState {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: ShopConfig, game_config: Config) -> Self {
        let shop = Shop::new(geng, assets, config);
        let mut state = Self::load(geng, assets, shop, game_config);
        state.shop.tier_rounds = 0;
        state
    }

    pub fn load(geng: &Geng, assets: &Rc<Assets>, mut shop: Shop, game_config: Config) -> Self {
        shop.money = 10.min(4 + shop.round as Money);
        shop.round += 1;
        shop.tier_rounds += 1;
        shop.reroll(true);
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            render_shop: render::RenderShop::new(vec2(1.0, 1.0), 0, 0, 0),
            render: render::Render::new(geng, assets, &game_config),
            time: 0.0,
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
        self.render
            .draw(&self.shop, &self.render_shop, self.time, framebuffer);
    }

    fn update(&mut self, delta_time: f64) {
        self.time += delta_time;
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown {
                position,
                button: MouseButton::Left,
            } => {
                let position = position.map(|x| x as f32);
                if let Some((interaction, layout)) = self.get_under_pos_mut(position) {
                    if let Some(layout) = layout {
                        layout.hovered = true;
                        layout.pressed = true;
                    }
                    match interaction {
                        Interaction::TierUp => self.shop.tier_up(),
                        Interaction::Reroll => self.shop.reroll(false),
                        Interaction::Go => self.transition = true,
                        Interaction::SellCard => {}
                        Interaction::Card(card) => {
                            self.drag_card(card, position);
                        }
                    }
                }
            }
            geng::Event::MouseUp {
                position,
                button: MouseButton::Left,
            } => {
                self.render_shop
                    .layout
                    .walk_widgets_mut(&mut |widget| widget.pressed = false);
                self.drag_stop();
            }
            geng::Event::MouseMove { position, .. } => {
                self.render_shop
                    .layout
                    .walk_widgets_mut(&mut |widget| widget.hovered = false);
                if let Some((_, Some(layout))) =
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
            clans: {
                calc_clan_members(
                    self.shop
                        .cards
                        .party
                        .iter()
                        .filter_map(|card| card.as_ref())
                        .map(|card| &card.unit),
                )
            },
            ..self.game_config.clone()
        };

        let round = self
            .assets
            .rounds
            .get(self.shop.round - 1)
            .unwrap_or_else(|| panic!("Failed to find round number: {}", self.shop.round))
            .clone();
        let game_state = Game::new(
            &self.geng,
            &self.assets,
            config,
            self.shop.clone(),
            round,
            false,
        );
        Some(geng::Transition::Switch(Box::new(game_state)))
    }
}

pub enum Interaction {
    TierUp,
    Reroll,
    Go,
    SellCard,
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
    /// The number of rounds that the shop has not been upgraded to the next tier.
    /// Once the shop is tiered up, that number is reset to 0.
    pub tier_rounds: usize,
    pub money: Money,
    pub available: Vec<(UnitType, UnitTemplate)>,
    pub cards: Cards,
    pub drag: Option<Drag>,
    pub lives: i32,
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
    ) -> Option<(Interaction, Option<&mut render::LayoutWidget>)> {
        let layout = &mut self.render_shop.layout;
        if let Some((index, layout)) = layout
            .shop_cards
            .iter_mut()
            .enumerate()
            .find(|(_, layout)| layout.position.contains(position))
        {
            return Some((Interaction::Card(CardState::Shop { index }), Some(layout)));
        }

        let world_pos = self
            .render
            .camera
            .screen_to_world(self.render.framebuffer_size, position);
        for x in 0..MAX_PARTY {
            let pos = Position {
                side: Faction::Player,
                x: x as Coord,
            };
            let delta = pos.to_world_f32() - world_pos;
            if delta.len() < 0.5 {
                return Some((Interaction::Card(CardState::Party { index: x }), None));
            }
        }

        if let Some((index, layout)) = layout
            .inventory_cards
            .iter_mut()
            .enumerate()
            .find(|(_, layout)| layout.position.contains(position))
        {
            return Some((
                Interaction::Card(CardState::Inventory { index }),
                Some(layout),
            ));
        }

        if layout.tier_up.position.contains(position) {
            return Some((Interaction::TierUp, Some(&mut layout.tier_up)));
        }
        if layout.reroll.position.contains(position) {
            return Some((Interaction::Reroll, Some(&mut layout.reroll)));
        }
        if layout.go.position.contains(position) {
            return Some((Interaction::Go, Some(&mut layout.go)));
        }
        if layout.sell.position.contains(position) {
            return Some((Interaction::SellCard, Some(&mut layout.sell)));
        }

        None
    }

    pub fn drag_card(&mut self, state: CardState, position: Vec2<f32>) {
        if self.shop.money < UNIT_COST {
            return;
        }
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
            self.drag_stop_impl(drag);
        }
        self.shop.cards.check_triples(&self.assets);
    }

    fn drag_stop_impl(&mut self, drag: Drag) {
        match drag.target {
            DragTarget::Card { card, old_state } => {
                if let Some((interaction, _)) = self.get_under_pos_mut(drag.position) {
                    match interaction {
                        Interaction::Card(state) => {
                            let from_shop = matches!(old_state, CardState::Shop { .. });
                            let to_shop = matches!(state, CardState::Shop { .. });
                            if let Some(target) = self.shop.cards.get_card_mut(&state) {
                                if let Some(unit) = target {
                                    self.shop.money -= UNIT_COST;
                                    unit.unit.level_up(card.unit.clone());
                                    return;
                                } else {
                                    if from_shop && !to_shop {
                                        // Moved from the shop
                                        self.shop.money -= UNIT_COST;
                                        *target = Some(card);
                                        return;
                                    } else {
                                        // Change placement
                                        *target = Some(card);
                                        return;
                                    }
                                }
                            }
                        }
                        Interaction::SellCard => {
                            if !matches!(old_state, CardState::Shop { .. }) {
                                // Sell the card
                                self.shop.money += UNIT_SELL_COST;
                                return;
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

impl Shop {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: ShopConfig) -> Self {
        let units = assets
            .units
            .iter()
            .filter(|unit| unit.1.tier > 0)
            .map(|(name, unit)| (name.clone(), unit.clone()))
            .collect();
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            round: 0,
            tier: 1,
            tier_rounds: 0,
            money: 0,
            cards: Cards::new(),
            drag: None,
            available: units,
            config,
            lives: MAX_LIVES,
        }
    }

    pub fn tier_up(&mut self) {
        if let Some(cost) = tier_up_cost(self.tier, self.tier_rounds) {
            if self.money >= cost {
                self.tier += 1;
                self.tier_rounds = 0;
                self.money -= cost;
            }
        }
    }

    pub fn replace_party(&mut self, party: Vec<Unit>) {
        for unit in party {
            for index in 0..self.cards.party.len() {
                if let Some(card) = self.cards.party.get(index).expect("Slot must exist") {
                    if card.unit.id == unit.id {
                        if let Some(card_mut) = self.cards.party.get_mut(index).unwrap() {
                            card_mut.unit = unit.clone();
                        }
                    }
                }
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
                let mut rng = global_rng();
                let options = self
                    .available
                    .iter()
                    .filter(|(_, unit)| unit.tier <= self.tier)
                    .map(|(unit_type, unit)| {
                        Some(UnitCard::new(
                            unit.clone(),
                            unit_type.clone(),
                            &self.assets.statuses,
                        ))
                    })
                    .collect::<Vec<_>>();
                if options.is_empty() {
                    self.cards.shop = vec![];
                    error!("No units are available to roll");
                    return;
                }
                self.cards.shop = (0..units)
                    .map(|_| options.choose(&mut rng).unwrap().clone())
                    .collect();
            }
        }
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
        match *state {
            CardState::Shop { index } => self.shop.get_mut(index),
            CardState::Party { index } => self.party.get_mut(index),
            CardState::Inventory { index } => self.inventory.get_mut(index),
        }
    }

    pub fn check_triples(&mut self, assets: &Rc<Assets>) {
        // Count cards
        let mut counters = HashMap::new();
        for unit_type in self
            .party
            .iter()
            .chain(self.inventory.iter())
            .filter_map(|card| card.as_ref())
            .map(|card| card.unit.unit_type.clone())
        {
            *counters.entry(unit_type).or_insert(0) += 1;
        }
        counters.retain(|_, counter| *counter >= 3); // Remove unneeded counters
    }
}

impl Default for Cards {
    fn default() -> Self {
        Self::new()
    }
}

fn calc_clan_members<'a>(units: impl IntoIterator<Item = &'a Unit>) -> HashMap<Clan, usize> {
    let unique_units = units
        .into_iter()
        .map(|unit| (&unit.unit_type, &unit.clans))
        .collect::<HashMap<_, _>>();
    let mut clans = HashMap::new();
    for clan in unique_units.into_values().flatten() {
        *clans.entry(*clan).or_insert(0) += 1;
    }
    clans
}

fn roll(choices: &[UnitTemplate], tier: Tier, units: usize) -> Vec<UnitTemplate> {
    choices
        .iter()
        .filter(|unit| unit.tier <= tier)
        .cloned() // TODO: optimize
        .choose_multiple(&mut global_rng(), units)
}

fn earn_money(round: usize) -> Money {
    (4 + round).min(10) as _
}

fn tier_up_cost(current_tier: Tier, tier_rounds: usize) -> Option<Money> {
    TIER_UP_COST
        .get(current_tier as usize - 1)
        .map(|&cost| cost.saturating_sub(tier_rounds as Money))
}

fn tier_units_number(current_tier: Tier) -> Option<usize> {
    TIER_UNITS.get(current_tier as usize - 1).copied()
}
