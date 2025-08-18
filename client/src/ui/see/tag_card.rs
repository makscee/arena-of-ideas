use super::*;

pub trait SFnTagCard: SFnTag + SFnCard + Node {
    fn see_tag_card(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let expanded_id = self.egui_id().with(ui.id()).with("expanded");
        let expanded = ui
            .ctx()
            .data(|r| r.get_temp::<bool>(expanded_id))
            .unwrap_or(false);

        context.with_layer_ref_r(ContextLayer::Owner(self.entity()), |context| {
            let response = if expanded {
                self.see_card(context, ui)?
            } else {
                self.see_tag(context, ui)
            };
            if response.clicked() {
                ui.ctx().data_mut(|w| w.insert_temp(expanded_id, !expanded));
            }
            Ok(())
        })
    }
}

impl SFnTagCard for NUnit {}
impl SFnTagCard for NHouse {}
impl SFnTagCard for NActionAbility {}
impl SFnTagCard for NStatusAbility {}
