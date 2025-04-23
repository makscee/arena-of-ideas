use super::*;

pub trait NodeView: Node + NodeExt + ToCstr + DataView {
    fn node_title(&self, _context: &Context, ui: &mut Ui) -> Response {
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
    fn replace_context_menu(&mut self, context: &Context, ui: &mut Ui) -> Option<Self> {
        ui.menu_button("replace", |ui| node_menu(ui, context))
            .inner?
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

impl NodeView for NCore {}
impl NodeView for NPlayers {}
impl NodeView for NPlayer {}
impl NodeView for NPlayerData {}
impl NodeView for NPlayerIdentity {}
impl NodeView for NHouse {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.house_name, context, ui)
    }
}
impl NodeView for NHouseColor {
    fn node_title(&self, _context: &Context, ui: &mut Ui) -> Response {
        format!("{}{}", self.cstr(), self.color.cstr()).button(ui)
    }
}
impl NodeView for NAbilityMagic {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.ability_name, context, ui)
    }
}
impl NodeView for NAbilityDescription {}
impl NodeView for NAbilityEffect {}
impl NodeView for NStatusMagic {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.status_name, context, ui)
    }
}
impl NodeView for NStatusDescription {}
impl NodeView for NTeam {}
impl NodeView for NMatch {}
impl NodeView for NShopCaseUnit {}
impl NodeView for NFusion {}
impl NodeView for NUnit {
    fn node_title(&self, context: &Context, ui: &mut Ui) -> Response {
        colored_name(self, &self.unit_name, context, ui)
    }
}
impl NodeView for NUnitDescription {}
impl NodeView for NUnitStats {}
impl NodeView for NFusionStats {}
impl NodeView for NBehavior {}
impl NodeView for NRepresentation {}
impl NodeView for NArena {}
impl NodeView for NFloorPool {}
impl NodeView for NFloorBoss {}
impl NodeView for NBattle {}
