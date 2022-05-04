pub mod render;
mod ui;
mod unit_card;

use super::*;
use crate::render::UnitRender;

use unit_card::*;

const UNIT_COST: Money = 3;
const MAX_PARTY: usize = 7;
const MAX_INVENTORY: usize = 10;

pub struct ShopState {
    pub shop: Shop,
    pub render_shop: render::RenderShop,
    pub render: render::Render,
}

impl ShopState {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, config: ShopConfig) -> Self {
        Self {
            shop: Shop::new(geng, assets, config.units.map.values().cloned()), // TODO: possibly optimize
            render_shop: render::RenderShop::new(),
            render: render::Render::new(geng, assets, config),
        }
    }
}

impl geng::State for ShopState {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.render.draw(&self.shop, &self.render_shop, framebuffer);
    }

    fn update(&mut self, delta_time: f64) {
        self.render_shop.update(delta_time as _);
    }
}

pub type Money = u32;

pub struct Shop {
    pub geng: Geng,
    pub assets: Rc<Assets>,
    pub tier: Tier,
    pub frozen: bool,
    pub available: Vec<Option<UnitCard>>,
    pub party: Vec<Option<UnitCard>>,
    pub inventory: Vec<Option<UnitCard>>,
    pub dragging: Option<Dragging>,
}

pub enum Dragging {
    ShopCard(UnitCard, usize),
    PartyCard(UnitCard, usize),
    InventoryCard(UnitCard, usize),
}

impl Shop {
    pub fn new(
        geng: &Geng,
        assets: &Rc<Assets>,
        units: impl Iterator<Item = UnitTemplate>,
    ) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            tier: Tier::new(1).unwrap(),
            frozen: false,
            available: units.map(|unit| Some(UnitCard::new(unit))).collect(),
            party: vec![],
            inventory: vec![],
            dragging: None,
        }
    }

    pub fn drag_shop_unit(&mut self, index: usize) {
        self.drag_stop();
        if let Some(unit) = self.available.get_mut(index).and_then(|unit| unit.take()) {
            self.dragging = Some(Dragging::ShopCard(unit, index));
        }
    }

    pub fn drag_party_unit(&mut self, index: usize) {
        self.drag_stop();
        if let Some(unit) = self.party.get_mut(index).and_then(|unit| unit.take()) {
            self.dragging = Some(Dragging::PartyCard(unit, index));
        }
    }

    pub fn drag_inventory_unit(&mut self, index: usize) {
        self.drag_stop();
        if let Some(unit) = self.inventory.get_mut(index).and_then(|unit| unit.take()) {
            self.dragging = Some(Dragging::InventoryCard(unit, index));
        }
    }

    pub fn drag_stop(&mut self) {
        if let Some(dragging) = self.dragging.take() {
            match dragging {
                Dragging::ShopCard(unit, index) => {
                    *self.available.get_mut(index).unwrap() = Some(unit)
                }
                Dragging::PartyCard(unit, index) => {
                    *self.party.get_mut(index).unwrap() = Some(unit)
                }
                Dragging::InventoryCard(unit, index) => {
                    *self.inventory.get_mut(index).unwrap() = Some(unit)
                }
            }
        }
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
    TIER_UP.get(usize::from(current_tier) - 1).copied()
}

fn tier_units_number(current_tier: Tier) -> Option<usize> {
    TIER_UNITS.get(usize::from(current_tier) - 1).copied()
}
