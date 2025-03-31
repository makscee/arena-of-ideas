use super::*;

fn show_frame(
    id: Id,
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
        const M: i8 = 2;
        ui.vertical(|ui| {
            let response = ui
                .horizontal(|ui| {
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
                                .cstr_c(ui.visuals().faint_bg_color)
                                .as_label(ui.style())
                                .sense(Sense::click())
                                .ui(ui)
                        })
                        .inner;
                    show_state_btns(id, ui);
                    response
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

fn show_state_btns(id: Id, ui: &mut Ui) {
    let mut state = get_state(id, ui);
    let mut changed = false;
    let size = 6.0;
    let size = egui::vec2(size, size);
    if RectButton::new(size)
        .active(matches!(state.mode, NodeViewMode::Compact))
        .ui(ui, |color, rect, ui| {
            ui.painter()
                .line_segment([rect.left_bottom(), rect.right_bottom()], color.stroke());
        })
        .clicked()
    {
        state.mode = NodeViewMode::Compact;
        changed = true;
    }
    if RectButton::new(size)
        .active(matches!(state.mode, NodeViewMode::Full))
        .ui(ui, |color, rect, ui| {
            ui.painter()
                .line_segment([rect.left_center(), rect.right_center()], color.stroke());
            ui.painter()
                .line_segment([rect.center_top(), rect.center_bottom()], color.stroke());
        })
        .clicked()
    {
        state.mode = NodeViewMode::Full;
        changed = true;
    }
    if RectButton::new(size)
        .active(matches!(state.mode, NodeViewMode::Graph))
        .ui(ui, |color, rect, ui| {
            let rect = rect.shrink(1.0);
            ui.painter()
                .circle_stroke(rect.left_center(), 1.0, color.stroke());
            ui.painter()
                .circle_stroke(rect.right_top(), 1.0, color.stroke());
            ui.painter()
                .circle_stroke(rect.right_bottom(), 1.0, color.stroke());
        })
        .clicked()
    {
        state.mode = NodeViewMode::Graph;
        changed = true;
    }
    ui.add_space(1.0);
    if changed {
        set_state(id, state, ui);
    }
}

fn get_state(id: Id, ui: &mut Ui) -> NodeViewState {
    ui.ctx()
        .data(|r| r.get_temp::<NodeViewState>(id))
        .unwrap_or_default()
}
fn set_state(id: Id, state: NodeViewState, ui: &mut Ui) {
    ui.ctx().data_mut(|w| w.insert_temp(id, state))
}

#[derive(Clone, Copy, Debug, Default)]
pub enum NodeViewMode {
    #[default]
    Compact,
    Full,
    Graph,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeViewState {
    mode: NodeViewMode,
}
pub trait NodeView: NodeExt {
    fn view_id(&self) -> Id {
        Id::new(self.get_entity())
            .with(self.get_id())
            .with(self.kind())
    }
    fn view(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let mut context = context.clone();
        for (var, value, kind) in self.get_vars(&context) {
            context.set_var(var, value, kind);
        }
        let state = get_state(self.view_id(), ui);
        match state.mode {
            NodeViewMode::Compact => self.compact(ui, &context),
            NodeViewMode::Full => self.full(ui, &context),
            NodeViewMode::Graph => self.full(ui, &context),
        }
    }
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let title = self.kind().to_string();
        let color = context
            .get_color_any(VarName::color)
            .unwrap_or(ui.visuals().weak_text_color());
        show_frame(self.view_id(), &title, color, ui, |ui| {
            for (var, value) in self.get_own_vars() {
                ui.horizontal(|ui| {
                    var.cstr().label(ui);
                    value.cstr().label(ui);
                });
            }
        });
        Ok(())
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        self.compact(ui, context)
    }
}

pub trait NodeGraphView: NodeExt + NodeView {
    fn graph_view_self(&self, parent: Rect, context: &Context, ui: &mut Ui) -> Rect {
        let (rect, _) = show_frame(
            self.view_id(),
            &self.kind().cstr(),
            ui.visuals().text_color(),
            ui,
            |ui| {
                ui.vertical(|ui| {
                    self.show(None, context, ui);
                });
            },
        );
        ui.painter().line(
            [parent.right_center(), rect.left_center()].into(),
            ui.visuals().weak_text_color().stroke(),
        );
        rect
    }
    fn graph_view_self_mut(&mut self, parent: Rect, ui: &mut Ui) -> (bool, Rect) {
        let mut changed = false;
        let (rect, _) = show_frame(
            self.view_id(),
            &self.kind().cstr(),
            ui.visuals().text_color(),
            ui,
            |ui| {
                ui.vertical(|ui| {
                    changed = self.show_mut(None, ui);
                });
            },
        );
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

impl NodeView for All {}
impl NodeView for Incubator {}
impl NodeView for Player {}
impl NodeView for PlayerData {}
impl NodeView for PlayerIdentity {}
impl NodeView for House {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let name = context.get_string(VarName::name, NodeKind::House)?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        show_frame(self.view_id(), &name, color, ui, |ui| {
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
        });
        Ok(())
    }
}
impl NodeView for HouseColor {}
impl NodeView for ActionAbility {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let name = context.get_string(VarName::name, self.kind())?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        ui.horizontal(|ui| {
            TagWidget::new_name(name, color).ui(ui);
            show_state_btns(self.view_id(), ui);
        });
        Ok(())
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let house = context.get_string(VarName::name, NodeKind::House)?;
        show_frame(
            self.view_id(),
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
        );
        Ok(())
    }
}
impl NodeView for ActionAbilityDescription {}
impl NodeView for AbilityEffect {}
impl NodeView for StatusAbility {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let name = context.get_string(VarName::name, self.kind())?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        ui.horizontal(|ui| {
            TagWidget::new_name(name, color).ui(ui);
            show_state_btns(self.view_id(), ui);
        });
        Ok(())
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let house = context.get_string(VarName::name, NodeKind::House)?;
        show_frame(
            self.view_id(),
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
        );
        Ok(())
    }
}
impl NodeView for StatusAbilityDescription {}
impl NodeView for Team {}
impl NodeView for Match {}
impl NodeView for ShopCaseUnit {}
impl NodeView for Fusion {}
impl NodeView for Unit {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let name = context.get_string(VarName::name, self.kind())?;
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let pwr = context.get_i32(VarName::pwr, NodeKind::UnitStats)?;
        let hp = context.get_i32(VarName::hp, NodeKind::UnitStats)?;
        let stats = format!("[yellow {}]/[red {}]", pwr, hp);
        ui.horizontal(|ui| {
            TagWidget::new_name_value(name, color, stats).ui(ui);
            show_state_btns(self.view_id(), ui);
        });
        Ok(())
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let color = context.get_color(VarName::color, NodeKind::HouseColor)?;
        let pwr = context.get_i32(VarName::pwr, NodeKind::UnitStats)?;
        let hp = context.get_i32(VarName::hp, NodeKind::UnitStats)?;
        let house = context.get_string(VarName::name, NodeKind::House)?;
        show_frame(
            self.view_id(),
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
        );
        Ok(())
    }
}
impl NodeView for UnitDescription {}
impl NodeView for UnitStats {}
impl NodeView for Behavior {}
impl NodeView for Representation {}
