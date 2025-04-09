use super::*;

pub trait NodeView: Node + NodeExt + ToCstr {
    fn show_title(&self, context: &Context, ui: &mut Ui) -> Response {
        self.cstr().button(ui)
    }
}

impl NodeView for Core {}
impl NodeView for Players {}
impl NodeView for Incubator {}
impl NodeView for Player {}
impl NodeView for PlayerData {}
impl NodeView for PlayerIdentity {}
impl NodeView for House {}
impl NodeView for HouseColor {}
impl NodeView for AbilityMagic {}
impl NodeView for AbilityDescription {}
impl NodeView for AbilityEffect {}
impl NodeView for StatusMagic {}
impl NodeView for StatusDescription {}
impl NodeView for Team {}
impl NodeView for Match {}
impl NodeView for ShopCaseUnit {}
impl NodeView for Fusion {}
impl NodeView for Unit {}
impl NodeView for UnitDescription {}
impl NodeView for UnitStats {}
impl NodeView for Behavior {}
impl NodeView for Representation {}
