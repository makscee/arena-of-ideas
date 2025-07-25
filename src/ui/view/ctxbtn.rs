use super::*;

#[derive(Debug, PartialEq)]
pub enum CtxBtnAction<T> {
    Delete,
    Paste(T),
}

pub struct CtxBtnBuilder<'a, T: Clone> {
    item: &'a T,
    copy_action: Option<Box<dyn FnOnce(T) + 'a>>,
    paste_fn: Option<Box<dyn Fn() -> Option<T> + 'a>>,
    enable_delete: bool,
    custom_actions: Vec<(String, Box<dyn FnOnce(T) + 'a>)>,
    custom_dangerous_actions: Vec<(String, Box<dyn FnOnce(T) + 'a>)>,
}

impl<'a, T: Clone> CtxBtnBuilder<'a, T> {
    pub fn new(item: &'a T) -> Self {
        Self {
            item,
            copy_action: None,
            paste_fn: None,
            enable_delete: false,
            custom_actions: Vec::new(),
            custom_dangerous_actions: Vec::new(),
        }
    }

    pub fn add_copy(mut self) -> Self
    where
        T: StringData,
    {
        self.copy_action = Some(Box::new(|item| {
            clipboard_set(item.get_data());
        }));
        self
    }

    pub fn on_copy<F>(mut self, f: F) -> Self
    where
        F: FnOnce(T) + 'a,
    {
        self.copy_action = Some(Box::new(f));
        self
    }

    pub fn add_paste<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Option<T> + 'a,
        T: StringData,
    {
        self.paste_fn = Some(Box::new(f));
        self
    }

    pub fn with_paste(mut self) -> Self
    where
        T: StringData + Default,
    {
        self.paste_fn = Some(Box::new(|| {
            clipboard_get().and_then(|data| {
                let mut item = T::default();
                item.inject_data(&data).ok().map(|_| item)
            })
        }));
        self
    }

    pub fn with_delete(mut self) -> Self {
        self.enable_delete = true;
        self
    }

    pub fn add_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T) + 'a,
    {
        self.custom_actions.push((name, Box::new(f)));
        self
    }

    pub fn add_dangerous_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T) + 'a,
    {
        self.custom_dangerous_actions.push((name, Box::new(f)));
        self
    }

    pub fn ui(mut self, vctx: ViewContext, context: &Context, ui: &mut Ui) -> CtxBtnResponse<T>
    where
        T: ViewFns,
    {
        let mut delete_action = None;
        let mut paste_action = None;

        let title_response = ui
            .horizontal(|ui| {
                let title_response = self.item.view_title(vctx, context, ui);

                let circle_size = 12.0;
                let circle_response = RectButton::new_size(egui::Vec2::splat(circle_size)).ui(
                    ui,
                    |color, rect, _response, ui| {
                        ui.painter()
                            .circle_filled(rect.center(), rect.width() * 0.5, color);
                    },
                );

                circle_response.bar_menu(|ui| {
                    ui.set_min_width(120.0);

                    if let Some(copy_fn) = self.copy_action.take() {
                        if ui.button("📋 Copy").clicked() {
                            copy_fn(self.item.clone());
                            ui.close_menu();
                        }
                    }

                    if let Some(paste_fn) = &self.paste_fn {
                        if ui.button("📋 Paste").clicked() {
                            if let Some(data) = paste_fn() {
                                paste_action = Some(CtxBtnAction::Paste(data));
                            }
                            ui.close_menu();
                        }
                    }

                    for (name, action) in self.custom_actions {
                        if ui.button(&name).clicked() {
                            action(self.item.clone());
                            ui.close_menu();
                            break;
                        }
                    }

                    if !self.custom_dangerous_actions.is_empty() || self.enable_delete {
                        ui.separator();
                    }

                    for (name, action) in self.custom_dangerous_actions {
                        if ui
                            .add(
                                egui::Button::new(&name)
                                    .fill(ui.visuals().error_fg_color.gamma_multiply(0.2)),
                            )
                            .clicked()
                        {
                            action(self.item.clone());
                            ui.close_menu();
                            break;
                        }
                    }

                    if self.enable_delete {
                        if ui
                            .add(
                                egui::Button::new("🗑 Delete")
                                    .fill(ui.visuals().error_fg_color.gamma_multiply(0.2)),
                            )
                            .clicked()
                        {
                            delete_action = Some(CtxBtnAction::Delete);
                            ui.close_menu();
                        }
                    }
                });

                title_response
            })
            .inner;

        CtxBtnResponse {
            response: title_response,
            delete_action,
            paste_action,
        }
    }
}

pub struct CtxBtnResponse<T> {
    pub response: Response,
    pub delete_action: Option<CtxBtnAction<T>>,
    pub paste_action: Option<CtxBtnAction<T>>,
}

impl<T> CtxBtnResponse<T> {
    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    pub fn deleted(&self) -> bool {
        matches!(self.delete_action, Some(CtxBtnAction::Delete))
    }

    pub fn pasted(&self) -> Option<&T> {
        if let Some(CtxBtnAction::Paste(ref data)) = self.paste_action {
            Some(data)
        } else {
            None
        }
    }
}

pub trait CtxBtn: ViewFns + Sized + Clone {
    fn ctxbtn(&self) -> CtxBtnBuilder<Self> {
        CtxBtnBuilder::new(self)
    }
}

impl<T: ViewFns + Clone> CtxBtn for T {}

impl<'a, T: NodeViewFns + Clone> CtxBtnBuilder<'a, T> {
    pub fn with_node_defaults(self) -> Self
    where
        T: StringData + Default,
    {
        self.add_copy().with_paste().with_delete()
    }
}

pub trait NodeCtxBtn: NodeViewFns + Sized + Clone {
    fn node_ctxbtn(&self) -> CtxBtnBuilder<Self>
    where
        Self: StringData + Default,
    {
        CtxBtnBuilder::new(self).with_node_defaults()
    }
}

impl<T: NodeViewFns + Clone + StringData + Default> NodeCtxBtn for T {}
