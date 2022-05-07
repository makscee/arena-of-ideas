use geng::ui;
use geng::{ui::*, Draw2d, Event};

use super::*;

mod button;
mod card;
mod cards;

use button::Button;
use card::*;
use cards::*;

const CARDS_SPACE_IN: f64 = 15.0;
const CARDS_SPACE_OUT: f64 = 10.0;

impl Shop {
    pub fn ui<'a>(&'a mut self, cx: &'a Controller) -> Box<dyn Widget + 'a> {
        let mut top_left: Vec<Box<dyn Widget>> = vec![];
        if let Some(cost) = tier_up_cost(self.tier) {
            let tier_up = Button::new(cx, &format!("Tier Up ({})", cost));
            if tier_up.was_clicked() {
                self.tier_up();
            }
            top_left.push(Box::new(tier_up));
        }

        let current_tier = Text::new(
            format!("Tier {}", self.tier),
            cx.theme().font.clone(),
            cx.theme().text_size,
            Color::WHITE,
        );
        top_left.push(Box::new(current_tier));

        let money_text = if self.money == 1 { "coin" } else { "coins" };
        let money_text = format!("{} {}", self.money, money_text);
        let money = Text::new(
            money_text,
            cx.theme().font.clone(),
            cx.theme().text_size,
            Color::WHITE,
        );
        top_left.push(Box::new(money));

        let shop = ui::column!(
            ui::row!(
                ui::column(top_left).align(vec2(0.5, 0.5)),
                CardsRow::new(
                    unit_cards(&self.geng, &self.assets, &self.shop, cx, self.time.as_f32()),
                    CARDS_SPACE_IN,
                    CARDS_SPACE_OUT
                )
            ),
            CardsRow::new(
                unit_cards(
                    &self.geng,
                    &self.assets,
                    &self.party,
                    cx,
                    self.time.as_f32()
                ),
                CARDS_SPACE_IN,
                CARDS_SPACE_OUT
            ),
            CardsRow::new(
                unit_cards(
                    &self.geng,
                    &self.assets,
                    &self.inventory,
                    cx,
                    self.time.as_f32()
                ),
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
    game_time: f32,
) -> Vec<Box<dyn Widget + 'a>> {
    cards
        .iter()
        .filter_map(|card| card.as_ref())
        .map(|card| {
            Box::new(UnitCardWidget::new(geng, assets, cx, card, game_time)) as Box<dyn Widget>
        })
        .collect()
}
