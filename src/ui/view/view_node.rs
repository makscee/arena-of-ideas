use super::*;

pub trait NodeViewFns: NodeExt {
    fn view_node(&self, vctx: ViewContextNew, context: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.title_cstr(vctx, context).label(ui);
            if let Some(f) = Self::fn_view_data() {
                f(self, vctx, context, ui);
            }
        });
    }
}

impl<T> ViewFns for T
where
    T: NodeExt,
{
    fn title_cstr(&self, vctx: ViewContextNew, context: &Context) -> Cstr {
        self.cstr()
    }
    fn fn_view_data() -> Option<fn(&Self, ViewContextNew, &Context, &mut Ui)> {
        Some(|s, _, context, ui| {
            s.show(context, ui);
        })
    }
    fn fn_view_data_mut(
    ) -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> ViewResponseNew> {
        Some(|s, _, context, ui| {
            let mut vr = ViewResponseNew::default();
            vr.changed = s.show_mut(context, ui);
            vr
        })
    }
    fn fn_view_context_menu_extra_mut(
    ) -> Option<fn(&mut Self, ViewContextNew, &Context, &mut Ui) -> ViewResponseNew> {
        Some(|s, _, _, ui| {
            let mut vr = ViewResponseNew::default();
            if "[red delete]".cstr().button(ui).clicked() {
                vr.delete_me = true;
                ui.close_menu();
            }
            vr
        })
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
