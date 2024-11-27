use super::*;

pub trait Show {
    fn show(&self, fade_in: f32, ui: &mut Ui, world: &mut World);
}
fn show_counter(c: u32, fade_in: f32, ui: &mut Ui) {
    format!("x{}", c)
        .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
        .label_alpha(fade_in, ui);
}

impl Show for TPlayer {
    fn show(&self, _: f32, ui: &mut Ui, _: &mut World) {
        text_dots_text(
            "name".cstr(),
            self.name.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold),
            ui,
        );
        text_dots_text("id".cstr(), self.id.to_string().cstr_c(VISIBLE_LIGHT), ui);
    }
}
