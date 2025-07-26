use super::*;

#[derive(Debug, PartialEq)]
pub enum CtxBtnAction<T> {
    Delete,
    Paste(T),
    PasteNodeFull(T),
}

pub struct CtxBtnBuilder<'a, T: Clone> {
    item: &'a T,
    copy_action: Option<Box<dyn FnOnce(T) + 'a>>,
    paste_fn: Option<Box<dyn Fn() -> Option<T> + 'a>>,
    copy_node_full_action: Option<Box<dyn FnOnce(T) + 'a>>,
    paste_node_full_fn: Option<Box<dyn Fn() -> Option<T> + 'a>>,
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
            copy_node_full_action: None,
            paste_node_full_fn: None,
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

    pub fn add_copy_node_full(mut self, context: &'a Context<'a>) -> Self
    where
        T: Node,
    {
        let context_ptr = context as *const Context<'a>;
        self.copy_node_full_action = Some(Box::new(move |item| {
            let context = unsafe { &*context_ptr };
            if let Some(entity) = item.get_entity() {
                if let Ok(packed) = T::pack_entity(context, entity) {
                    let packed_nodes = packed.pack();
                    if let Ok(serialized) = ron::to_string(&packed_nodes) {
                        clipboard_set(serialized);
                    }
                }
            }
        }));
        self
    }

    pub fn add_paste_node_full(mut self) -> Self
    where
        T: Node,
    {
        self.paste_node_full_fn = Some(Box::new(move || {
            clipboard_get().and_then(|data| {
                if let Ok(packed_nodes) = ron::from_str::<PackedNodes>(&data) {
                    let mut id = next_id();
                    let res = T::unpack_id(packed_nodes.root, &packed_nodes).map(|mut n| {
                        n.reassign_ids(&mut id);
                        n
                    });
                    set_next_id(id);
                    res
                } else {
                    None
                }
            })
        }));
        self
    }

    pub fn ui(mut self, ui: &mut Ui) -> CtxBtnResponse<T>
    where
        T: SFnTitle,
    {
        let mut delete_action = None;
        let mut paste_action = None;
        let mut paste_node_full_action = None;

        let title_response = ui
            .horizontal(|ui| {
                let title_response = self.item.see_title_cstr().button(ui);

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

                    if let Some(copy_node_full_fn) = self.copy_node_full_action.take() {
                        if ui.button("📦 Copy Node Full").clicked() {
                            copy_node_full_fn(self.item.clone());
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

                    if let Some(paste_node_full_fn) = &self.paste_node_full_fn {
                        if ui.button("📦 Paste Node Full").clicked() {
                            if let Some(data) = paste_node_full_fn() {
                                paste_node_full_action = Some(CtxBtnAction::PasteNodeFull(data));
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
            paste_node_full_action,
        }
    }
}

pub struct CtxBtnResponse<T> {
    pub response: Response,
    pub delete_action: Option<CtxBtnAction<T>>,
    pub paste_action: Option<CtxBtnAction<T>>,
    pub paste_node_full_action: Option<CtxBtnAction<T>>,
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

    pub fn pasted_node_full(&self) -> Option<&T> {
        if let Some(CtxBtnAction::PasteNodeFull(ref data)) = self.paste_node_full_action {
            Some(data)
        } else {
            None
        }
    }
}

impl<'a, T: Clone + SFnTitle> SeeBuilder<'a, T> {
    pub fn ctxbtn(self) -> CtxBtnBuilder<'a, T> {
        CtxBtnBuilder::new(self.data())
    }
}

impl<'a, T: NodeExt + Clone + SFnTitle> SeeBuilder<'a, T> {
    pub fn node_ctxbtn(self) -> CtxBtnBuilder<'a, T>
    where
        T: StringData + Default,
    {
        CtxBtnBuilder::new(self.data()).with_node_defaults()
    }

    pub fn node_ctxbtn_full(self) -> CtxBtnBuilder<'a, T>
    where
        T: StringData + Default + Node,
    {
        CtxBtnBuilder::new(self.data()).with_node_full_defaults(self.context())
    }
}

impl<'a, T: NodeExt + Clone> CtxBtnBuilder<'a, T> {
    pub fn with_node_defaults(self) -> Self
    where
        T: StringData + Default,
    {
        self.add_copy().with_paste().with_delete()
    }

    pub fn with_node_full_defaults(self, context: &'a Context<'a>) -> Self
    where
        T: Node + StringData + Default,
    {
        self.add_copy()
            .with_paste()
            .add_copy_node_full(context)
            .add_paste_node_full()
            .with_delete()
    }
}
