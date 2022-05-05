use geng::{ui::*, Draw2d};

use super::*;

mod card;
mod cards;

use card::*;
use cards::*;

const CARDS_SPACE_IN: f64 = 15.0;
const CARDS_SPACE_OUT: f64 = 10.0;

impl Shop {
    pub fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        let shop = ui::column!(
            CardsRow::new(
                unit_cards(&self.geng, &self.assets, &self.shop, cx),
                CARDS_SPACE_IN,
                CARDS_SPACE_OUT
            ),
            CardsRow::new(
                unit_cards(&self.geng, &self.assets, &self.party, cx),
                CARDS_SPACE_IN,
                CARDS_SPACE_OUT
            ),
            CardsRow::new(
                unit_cards(&self.geng, &self.assets, &self.inventory, cx),
                CARDS_SPACE_IN,
                CARDS_SPACE_OUT
            )
        );

        Box::new(shop)
    }
}

fn unit_cards<'a>(
    geng: &Geng,
    assets: &Rc<Assets>,
    cards: &'a [Option<UnitCard>],
    cx: &'a Controller,
) -> Vec<Box<dyn Widget + 'a>> {
    cards
        .iter()
        .filter_map(|card| card.as_ref())
        .map(|card| Box::new(UnitCardWidget::new(geng, assets, cx, card)) as Box<dyn Widget>)
        .collect()
}
