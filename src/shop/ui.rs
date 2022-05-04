use geng::ui::*;

use super::*;

impl Shop {
    pub fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        let shop = ui::column!(row(unit_cards(
            &self.geng,
            &self.assets,
            &self.available,
            cx
        )),);

        Box::new(shop)
    }
}

struct UnitCardWidget<'a> {
    pub unit_render: UnitRender,
    pub sense: &'a mut Sense,
    pub card: &'a UnitCard,
}

impl<'a> UnitCardWidget<'a> {
    pub fn new(geng: &Geng, assets: &Rc<Assets>, cx: &'a Controller, card: &'a UnitCard) -> Self {
        Self {
            sense: cx.get_state(),
            unit_render: UnitRender::new(geng, assets),
            card,
        }
    }
}

impl<'a> Widget for UnitCardWidget<'a> {
    fn calc_constraints(&mut self, cx: &ConstraintsContext) -> Constraints {
        Constraints {
            min_size: vec2(0.0, 0.0),
            flex: vec2(100.0, 100.0),
        }
    }

    fn sense(&mut self) -> Option<&mut Sense> {
        Some(self.sense)
    }

    fn update(&mut self, delta_time: f64) {}

    fn draw(&mut self, cx: &mut DrawContext) {
        self.unit_render.draw_unit(
            &self.card.unit,
            &self.card.template,
            None,
            self.card.game_time.as_f32(),
            &geng::PixelPerfectCamera,
            cx.framebuffer,
        );
    }

    fn handle_event(&mut self, event: &geng::Event) {}
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
