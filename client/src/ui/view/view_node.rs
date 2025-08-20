use super::*;

#[allow(unused)]
pub trait NodeViewFns: NodeExt + ViewFns {
    fn node_title_cstr(&self, vctx: ViewContext, context: &Context) -> Cstr {
        self.cstr()
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
impl NodeViewFns for NActionEffect {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.actions.iter().map(|r| r.cstr()).join("\n")
    }
}
impl NodeViewFns for NStatusAbility {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.status_name.cstr()
    }
}
impl NodeViewFns for NStatusDescription {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.description.cstr()
    }
}
impl NodeViewFns for NStatusBehavior {
    fn node_title_cstr(&self, _: ViewContext, _: &Context) -> Cstr {
        self.reactions.iter().map(|r| r.cstr()).join("\n")
    }
}
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
impl NodeViewFns for NFusionSlot {}
