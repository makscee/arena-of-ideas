use super::*;

pub fn show_unit_tag(context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
    TagWidget::new_name_value(
        context.get_string(VarName::unit_name)?,
        context.get_color(VarName::color)?,
        format!(
            "[b {} {}]",
            context.get_i32(VarName::pwr)?.cstr_c(VarName::pwr.color()),
            context.get_i32(VarName::hp)?.cstr_c(VarName::hp.color())
        ),
    )
    .ui(ui);
    Ok(())
}
