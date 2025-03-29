use super::*;

pub trait NodeView: NodeExt {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let mut context = context.clone();
        let vars: HashMap<VarName, VarValue> = HashMap::from_iter(self.get_own_vars());
        for (var, value) in &vars {
            context.set_var(*var, value.clone());
        }
        let title = self.kind().to_string();
        let color = context
            .get_color(VarName::color)
            .unwrap_or(tokens_global().ui_element_border_and_focus_rings());
        show_frame(&title, color, ui, |ui| {
            for (var, value) in vars {
                ui.horizontal(|ui| {
                    var.cstr().label(ui);
                    value.cstr().label(ui);
                });
            }
        });
        Ok(())
    }
    fn full(&self, ui: &mut Ui, context: &Context) {}
}

fn show_frame(title: &str, color: Color32, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    Frame {
        inner_margin: Margin::ZERO,
        outer_margin: MARGIN,
        fill: tokens_global().subtle_background(),
        stroke: color.stroke(),
        corner_radius: ROUNDING,
        shadow: Shadow::NONE,
    }
    .show(ui, |ui| {
        ui.style_mut().spacing.item_spacing.y = 0.0;
        const R: u8 = ROUNDING.ne;
        const M: i8 = 6;
        Frame::new()
            .corner_radius(CornerRadius {
                nw: R,
                ne: 0,
                sw: 0,
                se: R,
            })
            .fill(color)
            .inner_margin(Margin::symmetric(6, 0))
            .show(ui, |ui| {
                title.cstr_c(tokens_global().subtle_background()).label(ui)
            });
        Frame::new()
            .inner_margin(Margin {
                left: M,
                right: M,
                top: 0,
                bottom: 0,
            })
            .show(ui, content);
    });
}

impl NodeView for All {}
impl NodeView for Incubator {}
impl NodeView for Player {}
impl NodeView for PlayerData {}
impl NodeView for PlayerIdentity {}
impl NodeView for House {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let name = self
            .get_var(VarName::name, &context)
            .to_e("Name not found")?
            .get_string()?;
        let color = self
            .get_var(VarName::color, context)
            .to_e("Failed to get color")?
            .get_color()?;
        show_frame(&name, color, ui, |ui| {
            ui.horizontal(|ui| {
                "units:".cstr().label(ui);
                ui.vertical(|ui| {})
            });
        });
        Ok(())
    }
}
impl NodeView for HouseColor {}
impl NodeView for ActionAbility {}
impl NodeView for ActionAbilityDescription {}
impl NodeView for AbilityEffect {}
impl NodeView for StatusAbility {}
impl NodeView for StatusAbilityDescription {}
impl NodeView for Team {}
impl NodeView for Match {}
impl NodeView for ShopCaseUnit {}
impl NodeView for Fusion {}
impl NodeView for Unit {}
impl NodeView for UnitDescription {}
impl NodeView for UnitStats {}
impl NodeView for Behavior {}
impl NodeView for Representation {}

impl House {
    fn f(&self, context: &Context) {
        if let Some(d) = self.color.as_ref().or_else(|| {
            self.entity
                .and_then(|e| context.get_component::<HouseColor>(e))
        }) {}
    }
    fn c<'a>(&'a self, context: &'a Context) -> Option<&'a HouseColor> {
        self.color.as_ref().or_else(|| {
            self.entity
                .and_then(|e| context.get_component::<HouseColor>(e))
        })
    }
    fn cc<'a>(&'a self, context: &'a Context) -> Vec<&'a Unit> {
        if !self.units.is_empty() {
            self.units.iter().collect()
        } else if let Some(entity) = self.entity {
            context.children_components::<Unit>(entity)
        } else {
            default()
        }
    }
}
