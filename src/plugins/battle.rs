use super::*;

pub trait BattleRender {
    fn show_at(&mut self, t: f32, ui: &mut Ui);
}

impl BattleRender for BattleSimulation {
    fn show_at(&mut self, t: f32, ui: &mut Ui) {
        let center_rect = slot_rect(0, true, ui.available_rect_before_wrap(), self.slots);
        let up = center_rect.width() * 0.5;
        for (slot, side) in (1..=self.slots).cartesian_product([true, false]) {
            show_slot(self, slot, side, ui);
        }
        let unit_size = center_rect.width() * UNIT_SIZE;
        let mut q = self.world.query::<(Entity, &Representation)>();
        let context = Context::new_world(&self.world).set_t(t).take();
        for (e, rep) in q.iter(&self.world) {
            let context = context.clone().set_owner(e).take();
            let position = context
                .get_var(VarName::position)
                .unwrap_or_default()
                .get_vec2()
                .unwrap()
                .to_evec2()
                * up;
            let rect = Rect::from_center_size(
                center_rect.center() + position,
                egui::Vec2::splat(unit_size),
            );
            match RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui) {
                Ok(_) => {}
                Err(e) => error!("Rep paint error: {e}"),
            }
        }
    }
}

fn show_slot(bs: &BattleSimulation, i: usize, side: bool, ui: &mut Ui) -> Response {
    let full_rect = ui.available_rect_before_wrap();
    const FRAME: Frame = Frame {
        inner_margin: Margin::same(0.0),
        outer_margin: Margin::same(0.0),
        rounding: Rounding::ZERO,
        shadow: Shadow::NONE,
        fill: TRANSPARENT,
        stroke: STROKE_DARK,
    };
    let rect = slot_rect(i, side, full_rect, bs.slots);
    ui.expand_to_include_rect(rect);
    let mut cui = ui.child_ui(rect, *ui.layout(), None);
    let r = cui.allocate_rect(rect, Sense::hover());
    let stroke = if r.hovered() {
        STROKE_YELLOW
    } else {
        STROKE_DARK
    };
    cui.painter().add(FRAME.stroke(stroke).paint(r.rect));
    r
}

fn slot_rect(i: usize, side: bool, full_rect: Rect, team_slots: usize) -> Rect {
    let total_slots = team_slots * 2 + 1;
    let pos_i = if side {
        (team_slots - i) as i32
    } else {
        (team_slots + i) as i32
    } as f32;
    let size = (full_rect.width() / total_slots as f32).at_most(full_rect.height());
    let mut rect = full_rect;
    rect.set_height(size);
    rect.set_width(size);
    rect.translate(egui::vec2(size * pos_i, 0.0))
}
