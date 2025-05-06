use super::*;

pub trait NodeViewFns: NodeExt {
    fn view_node(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.node_title_cstr(vctx, context).label(ui);
            self.view_data(vctx, context, ui);
        });
    }
    fn node_title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
    }
    fn view_data(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        self.show(context, ui);
    }
    fn view_data_mut(&mut self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = ViewResponse::default();
        vr.changed = self.show_mut(context, ui);
        vr
    }
    fn view_context_menu_extra_mut(
        &mut self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> ViewResponse {
        default()
    }
}

impl NodeViewFns for NCore {}
impl NodeViewFns for NPlayers {}
impl NodeViewFns for NPlayer {}
impl NodeViewFns for NPlayerData {}
impl NodeViewFns for NPlayerIdentity {}
impl NodeViewFns for NHouse {}
impl NodeViewFns for NHouseColor {}
impl NodeViewFns for NAbilityMagic {}
impl NodeViewFns for NAbilityDescription {}
impl NodeViewFns for NAbilityEffect {}
impl NodeViewFns for NStatusMagic {}
impl NodeViewFns for NStatusDescription {}
impl NodeViewFns for NTeam {}
impl NodeViewFns for NMatch {}
impl NodeViewFns for NShopCaseUnit {}
impl NodeViewFns for NFusion {}
impl NodeViewFns for NUnit {}
impl NodeViewFns for NUnitDescription {}
impl NodeViewFns for NUnitStats {}
impl NodeViewFns for NBehavior {}
impl NodeViewFns for NRepresentation {}
impl NodeViewFns for NArena {}
impl NodeViewFns for NFloorPool {}
impl NodeViewFns for NFloorBoss {}
impl NodeViewFns for NBattle {}
