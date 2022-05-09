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
        // Top left
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

        // Top right
        let mut top_right: Vec<Box<dyn Widget>> = vec![];
        let reroll = Button::new(cx, "Reroll");
        if reroll.was_clicked() {
            self.reroll();
        }
        top_right.push(Box::new(reroll));

        let freeze = Button::new(cx, "Freeze");
        if freeze.was_clicked() {
            self.freeze();
        }
        top_right.push(Box::new(freeze));

        let mut shop_cards = vec![];
        let mut party_cards = vec![];
        let mut inventory_cards = vec![];
        let mut drag_card = None;
        for card in &mut self.cards {
            let mut widget = Box::new(UnitCardWidget::new(
                &self.geng,
                &self.assets,
                cx,
                Some(card),
                self.time.as_f32(),
            ));
            let card = widget.card.as_mut().unwrap();
            if widget.sense.is_captured() && drag_card.is_none() {
                card.state = CardState::Dragged {
                    old_state: Box::new(card.state.clone()),
                };
                drag_card = Some(widget as Box<dyn Widget>);
                continue;
            }

            match &card.state {
                CardState::Shop { .. } => {
                    shop_cards.push(widget as Box<dyn Widget>);
                }
                CardState::Party { .. } => {
                    party_cards.push(widget as Box<dyn Widget>);
                }
                CardState::Inventory { .. } => {
                    inventory_cards.push(widget as Box<dyn Widget>);
                }
                CardState::Dragged { old_state } => {
                    if !widget.sense.is_captured() {
                        card.state = (**old_state).clone();
                    }
                }
            }
        }

        let fix = |cards: &mut Vec<_>| {
            if cards.is_empty() {
                cards.push(Box::new(UnitCardWidget::new(
                    &self.geng,
                    &self.assets,
                    cx,
                    None,
                    self.time.as_f32(),
                )) as Box<dyn Widget>);
            }
        };
        fix(&mut shop_cards);
        fix(&mut party_cards);
        fix(&mut inventory_cards);

        let mut shop = ui::stack!(ui::column!(
            ui::row!(
                ui::column(top_left)
                    .align(vec2(0.5, 0.5))
                    .uniform_padding(5.0),
                CardsRow::new(shop_cards, CARDS_SPACE_IN, CARDS_SPACE_OUT).uniform_padding(10.0),
                ui::column(top_right)
                    .align(vec2(0.5, 0.5))
                    .uniform_padding(5.0),
            )
            .uniform_padding(5.0),
            CardsRow::new(party_cards, CARDS_SPACE_IN, CARDS_SPACE_OUT).uniform_padding(50.0),
            CardsRow::new(inventory_cards, CARDS_SPACE_IN, CARDS_SPACE_OUT).uniform_padding(5.0)
        )
        .uniform_padding(10.0));
        if let Some(dragged) = drag_card {
            shop.push(dragged);
        }

        Box::new(shop)
    }
}
