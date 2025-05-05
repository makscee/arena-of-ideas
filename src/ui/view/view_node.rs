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
