use super::*;

pub trait Show {
    fn show(&self, fade_in: f32, ui: &mut Ui, world: &mut World);
}
fn show_counter(c: u32, fade_in: f32, ui: &mut Ui) {
    format!("x{}", c)
        .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
        .label_alpha(fade_in, ui);
}
