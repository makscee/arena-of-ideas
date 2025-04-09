use super::*;

pub trait NodeView: Node + NodeExt + ToCstr {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        self.cstr().button(ui)
    }
    fn node_collapsed(&self, context: &Context, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(false, |ui| {
                self.node_title(context, ui);
            });
            "([tw ...])".cstr().button(ui)
        })
        .inner
    }
}

fn colored_name(node: &impl Node, name: &str, context: &Context, ui: &mut Ui) -> Response {
    let color = context
        .get_color(VarName::color)
        .unwrap_or(ui.visuals().text_color());
    format!("[tw {}] [b {}]", node.kind(), name)
        .cstr_c(color)
        .button(ui)
}

impl NodeView for Core {}
impl NodeView for Players {}
impl NodeView for Incubator {}
impl NodeView for Player {}
impl NodeView for PlayerData {}
impl NodeView for PlayerIdentity {}
impl NodeView for House {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.house_name, context, ui)
    }
}
impl NodeView for HouseColor {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        format!("{}{}", self.cstr(), self.color.cstr()).button(ui)
    }
}
impl NodeView for AbilityMagic {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.ability_name, context, ui)
    }
}
impl NodeView for AbilityDescription {}
impl NodeView for AbilityEffect {}
impl NodeView for StatusMagic {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.status_name, context, ui)
    }
}
impl NodeView for StatusDescription {}
impl NodeView for Team {}
impl NodeView for Match {}
impl NodeView for ShopCaseUnit {}
impl NodeView for Fusion {}
impl NodeView for Unit {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.unit_name, context, ui)
    }
}
impl NodeView for UnitDescription {}
impl NodeView for UnitStats {}
impl NodeView for Behavior {}
impl NodeView for Representation {}
