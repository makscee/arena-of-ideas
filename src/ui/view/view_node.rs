use super::*;

#[allow(unused)]
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
            self.view_data(vctx, context, ui);
        });
        vr
    }
    fn node_title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
    }
    fn node_rating(&self) -> Option<i32> {
        self.id().node_rating()
    }
    fn node_link_rating(
        &self,
        context: &Context,
        is_parent: bool,
        id: u64,
    ) -> Result<(i32, bool), ExpressionError> {
        let (child, parent) = if is_parent {
            (self.id(), id)
        } else {
            (id, self.id())
        };
        let (rating, solid) = context
            .world()?
            .get_any_link_rating(parent, child)
            .to_e_not_found()?;
        Ok((rating, solid))
    }
    fn node_view_link_rating(
        &self,
        vctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
        is_parent: bool,
        id: u64,
    ) {
        let (text, solid) = if let Ok((r, solid)) = self.node_link_rating(context, is_parent, id) {
            (r.cstr_expanded(), solid)
        } else {
            ("[tw _]".cstr(), false)
        };
        let (child, parent) = if is_parent {
            (self.id(), id)
        } else {
            (id, self.id())
        };
        rating_button(
            ui,
            text,
            solid,
            |ui| {
                "link rating vote".cstr().label(ui);
            },
            || {
                cn().reducers
                    .content_vote_link(parent, child, true)
                    .notify_error_op()
            },
            || {
                cn().reducers
                    .content_vote_link(parent, child, false)
                    .notify_error_op()
            },
        );
    }
    fn node_view_rating(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        let Some(r) = self.node_rating() else {
            "[red Node not found]".cstr().label(ui);
            return;
        };
        rating_button(
            ui,
            r.cstr_expanded(),
            false,
            |ui| {
                "node rating vote".cstr().label(ui);
            },
            || {
                cn().reducers
                    .content_vote_node(self.id(), true)
                    .notify_error_op();
            },
            || {
                cn().reducers
                    .content_vote_node(self.id(), false)
                    .notify_error_op();
            },
        );
    }
    fn view_data(&self, vctx: ViewContext, context: &Context, ui: &mut Ui) {
        // self.show(context, ui);
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

fn rating_button(
    ui: &mut Ui,
    text: String,
    active: bool,
    open: impl FnOnce(&mut Ui),
    minus: impl FnOnce(),
    plus: impl FnOnce(),
) {
    text.as_button().active(active, ui).ui(ui).bar_menu(|ui| {
        ui.vertical(|ui| {
            open(ui);
            ui.horizontal(|ui| {
                if "[red [b -]]".cstr().button(ui).clicked() {
                    plus()
                }
                if "[green [b +]]".cstr().button(ui).clicked() {
                    minus()
                }
            });
        });
    });
}

impl NodeViewFns for NCore {}
impl NodeViewFns for NPlayers {}
impl NodeViewFns for NPlayer {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.player_name.cstr()
    }
}
impl NodeViewFns for NPlayerData {}
impl NodeViewFns for NPlayerIdentity {}
impl NodeViewFns for NHouse {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.house_name.cstr()
    }
}
impl NodeViewFns for NHouseColor {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.color.cstr()
    }
}
impl NodeViewFns for NActionAbility {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.ability_name.cstr()
    }
}
impl NodeViewFns for NActionDescription {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.description.cstr()
    }
}
impl NodeViewFns for NActionEffect {}
impl NodeViewFns for NStatusAbility {}
impl NodeViewFns for NStatusDescription {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.description.cstr()
    }
}
impl NodeViewFns for NStatusBehavior {}
impl NodeViewFns for NStatusRepresentation {}
impl NodeViewFns for NTeam {}
impl NodeViewFns for NMatch {}
impl NodeViewFns for NFusion {}
impl NodeViewFns for NUnit {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.unit_name.cstr()
    }
}
impl NodeViewFns for NUnitDescription {
    fn view_data(&self, _vctx: ViewContext, _context: &Context, _ui: &mut Ui) {}
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.description.cstr()
    }
}
impl NodeViewFns for NUnitStats {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        format!(
            "[{} {}]/[{} {}]",
            VarName::pwr.color().to_hex(),
            self.pwr,
            VarName::hp.color().to_hex(),
            self.hp
        )
    }
}
impl NodeViewFns for NUnitState {}
impl NodeViewFns for NUnitBehavior {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.reactions.iter().map(|r| r.cstr()).join("\n")
    }
}
impl NodeViewFns for NUnitRepresentation {
    fn node_title_cstr(&self, _vctx: ViewContext, _context: &Context) -> Cstr {
        self.material.cstr_expanded()
    }
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
