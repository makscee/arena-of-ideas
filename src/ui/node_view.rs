use super::*;

#[derive(Clone, Copy, Default)]
pub enum ViewMode {
    #[default]
    Compact,
    Full,
    Graph,
}
#[derive(Clone, Copy)]
pub struct ViewContext {
    pub mode: ViewMode,
    pub parent_rect: Option<Rect>,
    pub color: Color32,
    pub is_mut: bool,
    pub hide_buttons: bool,
}

impl ViewContext {
    fn set_mut(mut self) -> Self {
        self.is_mut = true;
        self
    }
    pub fn compact() -> Self {
        Self {
            mode: ViewMode::Compact,
            ..default()
        }
    }
    pub fn graph() -> Self {
        Self {
            mode: ViewMode::Graph,
            ..default()
        }
    }
    pub fn full() -> Self {
        Self {
            mode: ViewMode::Full,
            ..default()
        }
    }
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
    pub fn hide_buttons(mut self) -> Self {
        self.hide_buttons = true;
        self
    }
}

impl Default for ViewContext {
    fn default() -> Self {
        Self {
            color: tokens_global().solid_backgrounds(),
            mode: default(),
            parent_rect: None,
            is_mut: false,
            hide_buttons: false,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct ViewState {
    mode: ViewMode,
    pub delete_me: bool,
}

impl ViewContext {
    fn merge_state(mut self, node: &impl NodeView, context: &Context, ui: &mut Ui) -> Self {
        if let Some(state) = node.get_state(ui) {
            self.mode = state.mode;
        }
        if let Some(color) = node
            .get_var(VarName::color, context)
            .and_then(|v| v.get_color().ok())
        {
            self.color = color;
        } else if let Ok(color) = context.get_color(VarName::color) {
            self.color = color;
        }
        self
    }
    pub fn show_parent_line(&self, ui: &mut Ui) {
        let Some(rect) = self.parent_rect else {
            return;
        };
        const OFFSET: egui::Vec2 = egui::vec2(0.0, 10.0);
        ui.painter().line_segment(
            [rect.right_top() + OFFSET, ui.cursor().left_top() + OFFSET],
            self.color.stroke(),
        );
    }
}

pub trait NodeView: NodeExt + NodeGraphViewNew + Clone {
    fn get_state(&self, ui: &mut Ui) -> Option<ViewState> {
        ui.ctx().data(|r| r.get_temp::<ViewState>(self.view_id()))
    }
    fn set_state(&self, state: ViewState, ui: &mut Ui) {
        ui.ctx().data_mut(|w| w.insert_temp(self.view_id(), state));
    }
    fn clear_state(&self, ui: &mut Ui) {
        ui.ctx()
            .data_mut(|w| w.remove_temp::<ViewState>(self.view_id()));
    }
    fn view(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) {
        let mut view_ctx = view_ctx.merge_state(self, context, ui);
        let context = &mut context.clone();
        for (var, value) in self.get_vars(context) {
            context.set_var(var, value);
        }
        match view_ctx.mode {
            ViewMode::Compact => self.compact_self(view_ctx, context, ui).ui(ui),
            ViewMode::Full => self.full_self(view_ctx, context, ui).ui(ui),
            ViewMode::Graph => {
                ui.horizontal(|ui| {
                    self.data_self(view_ctx, context, ui).ui(ui);
                    view_ctx.parent_rect = Some(ui.min_rect());
                    ui.vertical(|ui| self.view_children(view_ctx, context, ui));
                });
            }
        }
    }
    fn view_mut(&mut self, view_ctx: ViewContext, ui: &mut Ui, world: &mut World) -> bool {
        let mut view_ctx = view_ctx.merge_state(self, &default(), ui).set_mut();
        match view_ctx.mode {
            ViewMode::Compact | ViewMode::Full => self.data_self_mut(view_ctx, ui),
            ViewMode::Graph => {
                ui.horizontal(|ui| {
                    let changed = self.data_self_mut(view_ctx, ui);
                    view_ctx.parent_rect = Some(ui.min_rect());
                    ui.vertical(|ui| self.view_children_mut(view_ctx, ui, world))
                        .inner
                        || changed
                })
                .inner
            }
        }
    }
    fn data_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        show_frame(view_ctx.color, ui, |ui| {
            show_header(self, None, view_ctx, ui, |_| {});
            show_body(ui, |ui| self.show(None, context, ui));
        });
        Ok(())
    }
    fn compact_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        self.data_self(view_ctx, context, ui)
    }
    fn full_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        self.data_self(view_ctx, context, ui)
    }
    fn data_self_mut(&mut self, view_ctx: ViewContext, ui: &mut Ui) -> bool {
        let mut changed = false;
        show_frame(view_ctx.color, ui, |ui| {
            show_header(self, None, view_ctx, ui, |_| {});
            show_body(ui, |ui| {
                changed = self.show_mut(None, ui);
            });
        });
        changed
    }
    fn show_buttons(&self, view_ctx: ViewContext, ui: &mut Ui) {
        let state = self.get_state(ui);
        let mode = if let Some(state) = state {
            state.mode
        } else {
            view_ctx.mode
        };
        let mut state = state.unwrap_or_default();
        let mut changed = false;
        let size = 8.0;
        let size = egui::vec2(size, size);
        if RectButton::new(size)
            .active(matches!(mode, ViewMode::Compact))
            .ui(ui, |color, rect, ui| {
                let rect = rect.shrink(1.0);
                ui.painter()
                    .line_segment([rect.left_bottom(), rect.right_bottom()], color.stroke());
            })
            .clicked()
        {
            state.mode = ViewMode::Compact;
            changed = true;
        }
        if RectButton::new(size)
            .active(matches!(mode, ViewMode::Full))
            .ui(ui, |color, rect, ui| {
                let rect = rect.shrink(1.0);
                ui.painter()
                    .rect_stroke(rect, 0, color.stroke(), egui::StrokeKind::Middle);
            })
            .clicked()
        {
            state.mode = ViewMode::Full;
            changed = true;
        }
        if RectButton::new(size)
            .active(matches!(mode, ViewMode::Graph))
            .ui(ui, |color, rect, ui| {
                let rect = rect.shrink(2.0);
                ui.painter()
                    .circle_stroke(rect.left_center(), 1.0, color.stroke());
                ui.painter()
                    .circle_stroke(rect.right_top(), 1.0, color.stroke());
                ui.painter()
                    .circle_stroke(rect.right_bottom(), 1.0, color.stroke());
            })
            .clicked()
        {
            state.mode = ViewMode::Graph;
            changed = true;
        }
        RectButton::new(size)
            .ui(ui, |color, rect, ui| {
                ui.painter().circle_filled(rect.center_top(), 1.0, color);
                ui.painter().circle_filled(rect.center(), 1.0, color);
                ui.painter().circle_filled(rect.center_bottom(), 1.0, color);
            })
            .bar_menu(|ui| {
                ui.menu_button("publish to incubator", |ui| {
                    if "self".cstr().button(ui).clicked() {
                        let nodes = [self.to_tnode()].to_vec();
                        ui.close_menu();
                    }
                    if "full".cstr().button(ui).clicked() {
                        let node = self.clone();
                        op(move |world| {
                            IncubatorPlugin::set_publish_nodes(node, world);
                            Window::new("Incubator publish", |ui, world| {
                                IncubatorPlugin::pane_new_node(ui, world).ui(ui);
                            })
                            .center_anchor()
                            .expand()
                            .push(world);
                        });
                        ui.close_menu();
                    }
                });
                if view_ctx.is_mut && "delete".cstr_c(RED).button(ui).clicked() {
                    state.delete_me = true;
                    changed = true;
                    ui.close_menu();
                }
            });
        ui.add_space(1.0);
        if changed {
            self.set_state(state, ui);
        }
    }
}

pub trait NodeGraphViewNew: NodeExt {
    fn view_children(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui);
    fn view_children_mut(&mut self, view_ctx: ViewContext, ui: &mut Ui, world: &mut World) -> bool;
}

fn show_frame(color: Color32, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    Frame {
        inner_margin: Margin::ZERO,
        outer_margin: Margin::ZERO,
        fill: ui.visuals().faint_bg_color,
        stroke: color.stroke(),
        corner_radius: ROUNDING,
        shadow: Shadow::NONE,
    }
    .show(ui, |ui| ui.vertical(content));
}
fn show_header(
    node: &impl NodeView,
    title: Option<String>,
    view_ctx: ViewContext,
    ui: &mut Ui,
    content: impl FnOnce(&mut Ui),
) {
    const R: u8 = ROUNDING.ne;
    const M: i8 = 2;
    let title = title.unwrap_or_else(|| node.kind().to_string());
    ui.horizontal(|ui| {
        Frame::new()
            .corner_radius(CornerRadius {
                nw: R,
                ne: 0,
                sw: 0,
                se: R,
            })
            .fill(view_ctx.color)
            .inner_margin(Margin::symmetric(M, 0))
            .show(ui, |ui| {
                title
                    .cstr_cs(ui.visuals().faint_bg_color, CstrStyle::Bold)
                    .label(ui);
            });
        if !view_ctx.hide_buttons {
            node.show_buttons(view_ctx, ui);
        }
        content(ui);
    });
}
fn show_body(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    Frame::new().inner_margin(4).show(ui, content);
}
fn name_tag(
    node: &impl NodeView,
    var: VarName,
    view_ctx: ViewContext,
    context: &Context,
    ui: &mut Ui,
) -> Result<(), ExpressionError> {
    let color = view_ctx.color;
    let name = context.get_string(var)?;
    ui.horizontal(|ui| {
        TagWidget::new_name(name, color).ui(ui);
        node.show_buttons(view_ctx, ui);
    });
    Ok(())
}

impl NodeView for Core {}
impl NodeView for Incubator {}
impl NodeView for Players {}
impl NodeView for Player {}
impl NodeView for PlayerData {}
impl NodeView for PlayerIdentity {}
impl NodeView for House {
    fn compact_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        name_tag(self, VarName::house_name, view_ctx, context, ui)
    }
    fn full_self(
        &self,
        mut view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let color = view_ctx.color;
        let name = context.get_string(VarName::house_name)?;
        view_ctx.mode = ViewMode::Compact;
        show_frame(color, ui, |ui| {
            show_header(self, Some(name), view_ctx, ui, |_| {});
            show_body(ui, |ui| {
                ui.horizontal(|ui| {
                    "ability:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                    if let Some(ability) = self.action_ability_load(context) {
                        ui.vertical(|ui| {
                            ability.view(view_ctx, context, ui);
                        });
                    }
                });
                ui.horizontal(|ui| {
                    "status:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                    if let Some(status) = self.status_ability_load(context) {
                        ui.vertical(|ui| {
                            status.view(view_ctx, context, ui);
                        });
                    }
                });
                ui.horizontal(|ui| {
                    "units:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                    ui.vertical(|ui| {
                        for (i, unit) in self.units_load(context).into_iter().enumerate() {
                            ui.push_id(ui.id().with(i), |ui| {
                                unit.view(view_ctx, context, ui);
                            });
                        }
                    })
                });
            });
        });
        Ok(())
    }
}
impl NodeView for HouseColor {}
impl NodeView for AbilityMagic {
    fn compact_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        name_tag(self, VarName::ability_name, view_ctx, context, ui)
    }
    fn full_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let color = view_ctx.color;
        let house = context.get_string(VarName::house_name)?;
        let name = context.get_string(VarName::ability_name)?;
        show_frame(color, ui, |ui| {
            show_header(self, Some(name), view_ctx, ui, |_| {});
            show_body(ui, |ui| {
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
            });
        });
        Ok(())
    }
}
impl NodeView for AbilityDescription {}
impl NodeView for AbilityEffect {}
impl NodeView for StatusMagic {
    fn compact_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        name_tag(self, VarName::status_name, view_ctx, context, ui)
    }
    fn full_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let color = view_ctx.color;
        let house = context.get_string(VarName::house_name)?;
        let name = context.get_string(VarName::status_name)?;
        show_frame(color, ui, |ui| {
            show_header(self, Some(name), view_ctx, ui, |_| {});
            show_body(ui, |ui| {
                let mut tags = TagsWidget::new();
                tags.add_name(house, color);
                tags.ui(ui);
                if let Some(description) = self.description_load(context) {
                    description
                        .description
                        .cstr_c(ui.visuals().weak_text_color())
                        .label_w(ui);
                    if let Some(behavior) = description.behavior_load(context) {
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
            });
        });
        Ok(())
    }
}
impl NodeView for StatusDescription {}
impl NodeView for Team {}
impl NodeView for Match {}
impl NodeView for ShopCaseUnit {}
impl NodeView for Fusion {
    fn data_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let units = self.units(context)?;
        let name = units.iter().map(|u| &u.unit_name).join("").cstr();
        let mut behavior: Vec<(String, Vec<String>)> = default();
        let mut hp = 0;
        let mut pwr = 0;
        for unit in self.units(context)? {
            if let Some(stats) = unit
                .description_load(context)
                .and_then(|d| d.stats_load(context))
            {
                hp += stats.hp;
                pwr += stats.pwr;
            }
        }
        for (tr, ars) in &self.behavior {
            let trigger_str = self.get_trigger(tr, context)?.cstr();
            let mut actions_str: Vec<String> = default();
            for ar in ars {
                actions_str.push(self.get_action(ar, context)?.1.cstr());
            }
            behavior.push((trigger_str, actions_str));
        }
        show_frame(tokens_global().solid_backgrounds(), ui, |ui| {
            show_header(self, Some(name), view_ctx, ui, |_| {});
            show_body(ui, |ui| {
                ui.horizontal(|ui| {
                    TagWidget::new_var_value(VarName::pwr, pwr.into()).ui(ui);
                    TagWidget::new_var_value(VarName::hp, hp.into()).ui(ui);
                });
                for unit in units {
                    let color = context
                        .clone()
                        .set_owner(unit.entity())
                        .get_color(VarName::color)
                        .unwrap_or_default();
                    unit.view(view_ctx.color(color), context, ui);
                }
                for (trigger, actions) in behavior {
                    trigger.label(ui);
                    ui.horizontal(|ui| {
                        ui.add_space(5.0);
                        ui.vertical(|ui| {
                            for action in actions {
                                action.label(ui);
                            }
                        });
                        let rect = ui.min_rect().translate(egui::vec2(1.0, 0.0));
                        ui.painter().line_segment(
                            [rect.left_top(), rect.left_bottom()],
                            Stroke::new(2.0, YELLOW),
                        );
                    });
                }
            });
        });
        Ok(())
    }
}
impl NodeView for Unit {
    fn compact_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let color = view_ctx.color;
        let name = context.get_string(VarName::unit_name)?;
        let pwr = context.get_i32(VarName::pwr)?;
        let hp = context.get_i32(VarName::hp)?;
        let stats = format!("[yellow {}]/[red {}]", pwr, hp);
        ui.horizontal(|ui| {
            TagWidget::new_name_value(name, color, stats).ui(ui);
            if !view_ctx.hide_buttons {
                self.show_buttons(view_ctx, ui);
            }
        });
        Ok(())
    }
    fn full_self(
        &self,
        view_ctx: ViewContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let color = view_ctx.color;
        let pwr = context.get_i32(VarName::pwr)?;
        let hp = context.get_i32(VarName::hp)?;
        let house = context.get_string(VarName::house_name)?;
        let name = context.get_string(VarName::unit_name)?;
        show_frame(color, ui, |ui| {
            show_header(self, Some(name), view_ctx, ui, |_| {});
            show_body(ui, |ui| {
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
                    if let Some(behavior) = description.behavior_load(context) {
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
            });
        });
        Ok(())
    }
}
impl NodeView for UnitDescription {}
impl NodeView for UnitStats {}
impl NodeView for Behavior {}
impl NodeView for Representation {}
