use super::*;

pub trait NodeViewFns: NodeExt + ViewFns {
    fn view_node(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
        let mut vr = ViewResponse::default();
        ui.horizontal(|ui| {
            if vctx.selected {
                ui.painter().rect_filled(
                    ui.available_rect_before_wrap(),
                    0,
                    ui.visuals().widgets.hovered.bg_fill,
                );
            }
            vr.title_clicked = self.view_title(vctx, context, ui).clicked();
            self.node_view_rating(vctx, context, ui);
            self.id().label(ui);
            self.view_data(vctx, context, ui);
        });
        vr
    }
    fn node_title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
    }
    fn node_view_rating(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        let Some(node) = cn().db.nodes_world().id().find(&self.id()) else {
            "[red Node not found]".cstr().label(ui);
            return;
        };
        let r = node.rating;
        fn rating_text(r: i32) -> String {
            if r > 0 {
                format!("[b [green {r}]]")
            } else if r < 0 {
                format!("[b [red {r}]]")
            } else {
                format!("[b {r}]")
            }
            .cstr()
        }
        format!("[tw [s n:]]{}", rating_text(r))
            .button(ui)
            .bar_menu(|ui| {
                ui.vertical(|ui| {
                    "Node rating vote".cstr().label(ui);
                    ui.horizontal(|ui| {
                        if "[red [b -]]".cstr().button(ui).clicked() {
                            cn().reducers.content_vote_node(node.id, false).notify_op();
                        }
                        if "[green [b +]]".cstr().button(ui).clicked() {
                            cn().reducers.content_vote_node(node.id, true).notify_op();
                        }
                    });
                });
            });
        if let Some((parent, id)) = vctx.link_rating {
            if let Ok(world) = context.world() {
                let (parent, child) = if !parent {
                    (self.id(), id)
                } else {
                    (id, self.id())
                };
                let r_text = if let Some(r) = world.get_link_rating(parent, child) {
                    rating_text(r).cstr()
                } else {
                    "[tw _]".cstr()
                };
                format!("[s [tw l:]]{r_text}").button(ui).bar_menu(|ui| {
                    ui.vertical(|ui| {
                        "Link rating vote".cstr().label(ui);
                        ui.horizontal(|ui| {
                            if "[red [b -]]".cstr().button(ui).clicked() {
                                cn().reducers
                                    .content_vote_link(parent, child, false)
                                    .notify_op();
                            }
                            if "[green [b +]]".cstr().button(ui).clicked() {
                                cn().reducers
                                    .content_vote_link(parent, child, true)
                                    .notify_op();
                            }
                        });
                    });
                });
            }
        }
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
        let mut vr = ViewResponse::default();
        if ui.button("publish").clicked() {
            let mut pack = self.pack();
            op(move |world| {
                Window::new("publish node", move |ui, world| {
                    if "publish".cstr().button(ui).clicked() {
                        cn().reducers
                            .content_publish_node(to_ron_string(&pack))
                            .unwrap();
                        WindowPlugin::close_current(world);
                    }
                    Context::from_world(world, |context| {
                        pack.kind()
                            .to_kind()
                            .view_pack_with_children_mut(context, ui, &mut pack)
                            .ui(ui);
                    });
                })
                .expand()
                .push(world);
            });
            ui.close_menu();
        }
        ui.menu_button("replace", |ui| {
            if let Some(n) = node_menu::<Self>(ui, context) {
                *self = n;
                vr.changed = true;
            }
        });
        vr
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
impl NodeViewFns for NUnitDescription {
    fn view_data(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        if vctx.one_line {
            self.description.label_t(ui);
        } else {
            self.show(context, ui);
        }
    }
}
impl NodeViewFns for NUnitStats {}
impl NodeViewFns for NBehavior {
    fn view_data(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        if vctx.one_line {
            let s = self
                .reactions
                .iter()
                .map(|r| {
                    format!(
                        "{} ({})",
                        r.trigger.cstr(),
                        r.actions.iter().map(|a| a.cstr()).join(", ")
                    )
                })
                .join(" ");
            s.label_t(ui);
        } else {
            self.show(context, ui);
        }
    }
}
impl NodeViewFns for NRepresentation {
    fn view_data(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        if vctx.one_line {
            RectButton::new_size(LINE_HEIGHT.v2())
                .ui(ui, |_, rect, _, ui| {
                    RepresentationPlugin::paint_rect(rect, context, &self.material, ui).ui(ui);
                })
                .on_hover_ui(|ui| {
                    self.view_with_children(vctx.one_line(false), context, ui);
                });
        } else {
            self.show(context, ui);
        }
    }
}
impl NodeViewFns for NArena {}
impl NodeViewFns for NFloorPool {}
impl NodeViewFns for NFloorBoss {}
impl NodeViewFns for NBattle {}
