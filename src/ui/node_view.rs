use super::*;

pub trait NodeView: NodeExt {
    fn view(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let is_compact_id = ui.id();
        let is_compact = get_ctx_bool_id_default(ui.ctx(), is_compact_id, true);
        let mut context = context.clone();
        for (var, value, kind) in self.get_vars(&context) {
            context.set_var(var, value, kind);
        }
        if if is_compact {
            self.compact(ui, &context)
        } else {
            self.full(ui, &context)
        }?
        .clicked()
        {
            set_ctx_bool_id(ui.ctx(), is_compact_id, !is_compact);
        }
        Ok(())
    }
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let title = self.kind().to_string();
        let color = context
            .get_color_any(VarName::color)
            .unwrap_or(ui.visuals().weak_text_color());
        Ok(show_frame(&title, color, ui, |ui| {
            for (var, value) in self.get_own_vars() {
                ui.horizontal(|ui| {
                    var.cstr().label(ui);
                    value.cstr().label(ui);
                });
            }
        })
        .1)
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        self.compact(ui, context)
    }
}

pub trait NodeGraphView: NodeExt {
    fn graph_view_self(&self, parent: Rect, context: &Context, ui: &mut Ui) -> Rect {
        let (rect, _) = show_frame(&self.kind().cstr(), ui.visuals().text_color(), ui, |ui| {
            ui.vertical(|ui| {
                self.show(None, context, ui);
            });
        });
        ui.painter().line(
            [parent.right_center(), rect.left_center()].into(),
            ui.visuals().weak_text_color().stroke(),
        );
        rect
    }
    fn graph_view_self_mut(&mut self, parent: Rect, ui: &mut Ui) -> (bool, Rect) {
        let mut changed = false;
        let (rect, _) = show_frame(&self.kind().cstr(), ui.visuals().text_color(), ui, |ui| {
            ui.vertical(|ui| {
                changed = self.show_mut(None, ui);
            });
        });
        ui.painter().line(
            [parent.right_center(), rect.left_center()].into(),
            ui.visuals().weak_text_color().stroke(),
        );
        (changed, rect)
    }
    fn graph_view(&self, parent: Rect, context: &Context, ui: &mut Ui);
    fn graph_view_mut(&mut self, parent: Rect, ui: &mut Ui) -> bool;
    fn graph_view_mut_world(entity: Entity, parent: Rect, ui: &mut Ui, world: &mut World) -> bool;
}

fn show_frame(
    title: &str,
    color: Color32,
    ui: &mut Ui,
    content: impl FnOnce(&mut Ui),
) -> (Rect, Response) {
    let outer_margin = Margin::same(4);
    let response = Frame {
        inner_margin: Margin::ZERO,
        outer_margin,
        fill: ui.visuals().faint_bg_color,
        stroke: color.stroke(),
        corner_radius: ROUNDING,
        shadow: Shadow::NONE,
    }
    .show(ui, |ui| {
        const R: u8 = ROUNDING.ne;
        const M: i8 = 6;
        ui.vertical(|ui| {
            let response = Frame::new()
                .corner_radius(CornerRadius {
                    nw: R,
                    ne: 0,
                    sw: 0,
                    se: R,
                })
                .fill(color)
                .inner_margin(Margin {
                    left: M,
                    right: M,
                    top: 0,
                    bottom: 0,
                })
                .show(ui, |ui| {
                    title
                        .cstr_cs(ui.visuals().faint_bg_color, CstrStyle::Bold)
                        .as_label(ui.style())
                        .sense(Sense::click())
                        .ui(ui)
                })
                .inner;
            ui.push_id(title, |ui| {
                Frame::new()
                    .inner_margin(Margin {
                        left: M,
                        right: M,
                        top: 0,
                        bottom: M,
                    })
                    .show(ui, content);
            });
            response
        })
        .inner
    });
    (response.response.rect - outer_margin, response.inner)
}

impl NodeView for All {}
impl NodeView for Incubator {}
impl NodeView for Player {}
impl NodeView for PlayerData {}
impl NodeView for PlayerIdentity {}
impl NodeView for House {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let name = context.get_string(VarName::name, NodeKind::House)?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        Ok(show_frame(&name, color, ui, |ui| {
            ui.horizontal(|ui| {
                "ability:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                if let Some(ability) = self.action_ability_load(context) {
                    ui.vertical(|ui| {
                        ability.view(ui, context).ui(ui);
                    });
                }
            });
            ui.horizontal(|ui| {
                "status:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                if let Some(status) = self.status_ability_load(context) {
                    ui.vertical(|ui| {
                        status.view(ui, context).ui(ui);
                    });
                }
            });
            ui.horizontal(|ui| {
                "units:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                ui.vertical(|ui| {
                    for (i, unit) in self.units_load(context).into_iter().enumerate() {
                        ui.push_id(ui.id().with(i), |ui| {
                            unit.view(ui, context).ui(ui);
                        });
                    }
                })
            });
        })
        .1)
    }
}
impl NodeView for HouseColor {}
impl NodeView for ActionAbility {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let name = context.get_string(VarName::name, self.kind())?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        Ok(TagWidget::new_name(name, color).ui(ui))
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let house = context.get_string(VarName::name, NodeKind::House)?;
        Ok(show_frame(
            &context.get_string(VarName::name, self.kind())?,
            color,
            ui,
            |ui| {
                let mut tags = TagsWidget::new();
                tags.add_name(house, color);
                tags.ui(ui);
                if let Some(description) = self.description_load(context) {
                    description
                        .description
                        .cstr_c(ui.visuals().weak_text_color())
                        .label_w(ui);
                    if let Some(effect) = description.effect_load(context) {
                        ui.vertical(|ui| {
                            for action in &effect.actions.0 {
                                action.cstr().label(ui);
                            }
                        });
                    }
                }
            },
        )
        .1)
    }
}
impl NodeView for ActionAbilityDescription {}
impl NodeView for AbilityEffect {}
impl NodeView for StatusAbility {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let name = context.get_string(VarName::name, self.kind())?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        Ok(TagWidget::new_name(name, color).ui(ui))
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let house = context.get_string(VarName::name, NodeKind::House)?;
        Ok(show_frame(
            &context.get_string(VarName::name, self.kind())?,
            color,
            ui,
            |ui| {
                let mut tags = TagsWidget::new();
                tags.add_name(house, color);
                tags.ui(ui);
                if let Some(description) = self.description_load(context) {
                    description
                        .description
                        .cstr_c(ui.visuals().weak_text_color())
                        .label_w(ui);
                    if let Some(behavior) = description.reaction_load(context) {
                        for reaction in &behavior.reactions {
                            ui.vertical(|ui| {
                                reaction.trigger.cstr().label(ui);
                                for action in &reaction.actions.0 {
                                    action.cstr().label(ui);
                                }
                            });
                        }
                    }
                }
            },
        )
        .1)
    }
}
impl NodeView for StatusAbilityDescription {}
impl NodeView for Team {}
impl NodeView for Match {}
impl NodeView for ShopCaseUnit {}
impl NodeView for Fusion {}
impl NodeView for Unit {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let name = context.get_string(VarName::name, self.kind())?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let pwr = context.get_i32(VarName::pwr, NodeKind::UnitStats)?;
        let hp = context.get_i32(VarName::hp, NodeKind::UnitStats)?;
        let stats = format!("[yellow {}]/[red {}]", pwr, hp);
        Ok(TagWidget::new_name_value(name, color, stats).ui(ui))
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let pwr = context.get_i32(VarName::pwr, NodeKind::UnitStats)?;
        let hp = context.get_i32(VarName::hp, NodeKind::UnitStats)?;
        let house = context.get_string(VarName::name, NodeKind::House)?;
        Ok(show_frame(
            &context.get_string(VarName::name, NodeKind::Unit)?,
            color,
            ui,
            |ui| {
                let mut tags = TagsWidget::new();
                tags.add_name_value(VarName::pwr, VarName::pwr.color(), pwr.cstr());
                tags.add_name_value(VarName::hp, VarName::hp.color(), hp.cstr());
                tags.add_name(house, color);
                tags.ui(ui);
                if let Some(description) = self.description_load(context) {
                    description
                        .description
                        .cstr_c(ui.visuals().weak_text_color())
                        .label_w(ui);
                    if let Some(behavior) = description.reaction_load(context) {
                        for reaction in &behavior.reactions {
                            ui.vertical(|ui| {
                                reaction.trigger.cstr().label(ui);
                                for action in &reaction.actions.0 {
                                    action.cstr().label(ui);
                                }
                            });
                        }
                    }
                }
            },
        )
        .1)
    }
}
impl NodeView for UnitDescription {}
impl NodeView for UnitStats {}
impl NodeView for Behavior {}
impl NodeView for Representation {}
