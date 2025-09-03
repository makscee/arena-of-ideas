use super::*;

#[derive(Debug, PartialEq)]
pub enum CtxBtnAction<T> {
    Delete(T),
    Paste(T),
    PasteNodeFull(T),
}

pub enum CtxBtnItem<'a, T: Clone> {
    Action(
        String,
        Box<dyn FnOnce(T, &Context) -> Option<CtxBtnAction<T>> + 'a>,
    ),
    Submenu(String, Vec<CtxBtnItem<'a, T>>),
    Separator,
}

pub struct CtxBtnBuilder<'a, T: Clone> {
    item: &'a T,
    context: &'a Context<'a>,
    actions: Vec<CtxBtnItem<'a, T>>,
    dangerous_actions: Vec<CtxBtnItem<'a, T>>,
}

impl<'a, T: Clone> CtxBtnBuilder<'a, T> {
    pub fn new(item: &'a T, context: &'a Context<'a>) -> Self {
        Self {
            item,
            context,
            actions: Vec::new(),
            dangerous_actions: Vec::new(),
        }
    }

    pub fn add_copy(mut self) -> Self
    where
        T: StringData,
    {
        self.actions.push(CtxBtnItem::Action(
            "📋 Copy".to_string(),
            Box::new(|item, _context| {
                clipboard_set(item.get_data());
                None
            }),
        ));
        self
    }

    pub fn add_paste(mut self) -> Self
    where
        T: StringData + Default,
    {
        self.actions.push(CtxBtnItem::Action(
            "📋 Paste".to_string(),
            Box::new(|_item, _context| {
                clipboard_get()
                    .and_then(|data| {
                        let mut item = T::default();
                        item.inject_data(&data).ok().map(|_| item)
                    })
                    .map(|item| CtxBtnAction::Paste(item))
            }),
        ));
        self
    }

    pub fn add_copy_node_full(mut self) -> Self
    where
        T: Node,
    {
        self.actions.push(CtxBtnItem::Action(
            "📦 Copy Node Full".to_string(),
            Box::new(|item, context| {
                if let Some(entity) = item.get_entity() {
                    if let Ok(packed) = T::pack_entity(context, entity) {
                        let packed_nodes = packed.pack();
                        if let Ok(serialized) = ron::to_string(&packed_nodes) {
                            clipboard_set(serialized);
                        }
                    }
                }
                None
            }),
        ));
        self
    }

    pub fn add_paste_node_full(mut self) -> Self
    where
        T: Node,
    {
        self.actions.push(CtxBtnItem::Action(
            "📦 Paste Node Full".to_string(),
            Box::new(|_item, _context| {
                clipboard_get().and_then(|data| {
                    if let Ok(packed_nodes) = ron::from_str::<PackedNodes>(&data) {
                        let mut id = next_id();
                        let res = T::unpack_id(packed_nodes.root, &packed_nodes).map(|mut n| {
                            n.reassign_ids(&mut id);
                            n
                        });
                        set_next_id(id);
                        res.map(|item| CtxBtnAction::PasteNodeFull(item))
                    } else {
                        None
                    }
                })
            }),
        ));
        self
    }

    pub fn add_publish_submenu(mut self) -> Self
    where
        T: Node,
    {
        let publish_items = vec![
            CtxBtnItem::Action(
                "Node".to_string(),
                Box::new(|item: T, _context| {
                    let mut pack = PackedNodes::default();
                    pack.root = item.id();
                    item.pack_fill(&mut pack);

                    op(move |world| {
                        Window::new("Publish Node", move |ui, world| {
                            if "Publish".cstr().button(ui).clicked() {
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
                    None
                }),
            ),
            CtxBtnItem::Action(
                "Nested".to_string(),
                Box::new(|item: T, context| {
                    let mut pack = if let Some(entity) = item.get_entity() {
                        if let Ok(packed_item) = T::pack_entity(context, entity) {
                            packed_item.pack()
                        } else {
                            item.pack()
                        }
                    } else {
                        item.pack()
                    };

                    op(move |world| {
                        Window::new("Publish Node Nested", move |ui, world| {
                            if "Publish".cstr().button(ui).clicked() {
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
                    None
                }),
            ),
        ];

        self.actions
            .push(CtxBtnItem::Submenu("📤 Publish".to_string(), publish_items));
        self
    }

    pub fn add_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T, &Context) -> Option<CtxBtnAction<T>> + 'a,
    {
        self.actions.push(CtxBtnItem::Action(name, Box::new(f)));
        self
    }

    pub fn add_dangerous_action<F>(mut self, name: String, f: F) -> Self
    where
        F: FnOnce(T, &Context) -> Option<CtxBtnAction<T>> + 'a,
    {
        self.dangerous_actions
            .push(CtxBtnItem::Action(name, Box::new(f)));
        self
    }

    pub fn add_delete(mut self) -> Self {
        self.dangerous_actions.push(CtxBtnItem::Action(
            "🗑 Delete".to_string(),
            Box::new(|item, _context| Some(CtxBtnAction::Delete(item))),
        ));
        self
    }

    pub fn add_separator(mut self) -> Self {
        self.actions.push(CtxBtnItem::Separator);
        self
    }

    pub fn add_dangerous_separator(mut self) -> Self {
        self.dangerous_actions.push(CtxBtnItem::Separator);
        self
    }

    fn render_menu_items(
        items: Vec<CtxBtnItem<'a, T>>,
        item: &T,
        context: &Context,
        ui: &mut egui::Ui,
        dangerous: bool,
    ) -> Option<CtxBtnAction<T>> {
        let mut result = None;

        for menu_item in items {
            match menu_item {
                CtxBtnItem::Action(name, action) => {
                    let button = if dangerous {
                        ui.add(
                            egui::Button::new(&name)
                                .fill(ui.visuals().error_fg_color.gamma_multiply(0.2)),
                        )
                    } else {
                        ui.button(&name)
                    };

                    if button.clicked() {
                        result = action(item.clone(), context);
                        ui.close_menu();
                        break;
                    }
                }
                CtxBtnItem::Submenu(name, sub_items) => {
                    ui.menu_button(&name, |ui| {
                        if let Some(action) =
                            Self::render_menu_items(sub_items, item, context, ui, dangerous)
                        {
                            result = Some(action);
                        }
                    });
                }
                CtxBtnItem::Separator => {
                    ui.separator();
                }
            }
        }

        result
    }

    pub fn ui(self, ui: &mut Ui) -> CtxBtnResponse<T>
    where
        T: SFnTitle,
    {
        let mut action = None;

        let title_response = ui
            .horizontal(|ui| {
                let title_response = self.item.cstr_title().button(ui);

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

                    if let Some(result) =
                        Self::render_menu_items(self.actions, self.item, self.context, ui, false)
                    {
                        action = Some(result);
                    }

                    if !self.dangerous_actions.is_empty() {
                        ui.separator();
                    }

                    if let Some(result) = Self::render_menu_items(
                        self.dangerous_actions,
                        self.item,
                        self.context,
                        ui,
                        true,
                    ) {
                        action = Some(result);
                    }
                });

                title_response
            })
            .inner;

        CtxBtnResponse {
            response: title_response,
            action,
        }
    }
}

pub struct CtxBtnResponse<T> {
    pub response: Response,
    pub action: Option<CtxBtnAction<T>>,
}

impl<T> CtxBtnResponse<T> {
    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    pub fn action(&self) -> Option<&CtxBtnAction<T>> {
        self.action.as_ref()
    }

    pub fn deleted(&self) -> bool {
        matches!(self.action, Some(CtxBtnAction::Delete(_)))
    }

    pub fn pasted(&self) -> Option<&T> {
        if let Some(CtxBtnAction::Paste(data)) = &self.action {
            Some(data)
        } else {
            None
        }
    }

    pub fn pasted_node_full(&self) -> Option<&T> {
        if let Some(CtxBtnAction::PasteNodeFull(data)) = &self.action {
            Some(data)
        } else {
            None
        }
    }
}

impl<'a, T: Clone + SFnTitle> SeeBuilder<'a, T> {
    pub fn ctxbtn(self) -> CtxBtnBuilder<'a, T> {
        CtxBtnBuilder::new(self.data(), self.context())
    }
}

impl<'a, T: NodeExt + Clone + SFnTitle> SeeBuilder<'a, T> {
    pub fn node_ctxbtn(self) -> CtxBtnBuilder<'a, T>
    where
        T: StringData + Default,
    {
        CtxBtnBuilder::new(self.data(), self.context()).with_node_defaults()
    }

    pub fn node_ctxbtn_full(self) -> CtxBtnBuilder<'a, T>
    where
        T: StringData + Default + Node,
    {
        CtxBtnBuilder::new(self.data(), self.context()).with_node_full_defaults()
    }
}

impl<'a, T: NodeExt + Clone> CtxBtnBuilder<'a, T> {
    pub fn with_node_defaults(self) -> Self
    where
        T: StringData + Default,
    {
        self.add_copy().add_paste().add_delete()
    }

    pub fn with_node_full_defaults(self) -> Self
    where
        T: Node + StringData + Default,
    {
        self.add_copy()
            .add_paste()
            .add_copy_node_full()
            .add_paste_node_full()
            .add_publish_submenu()
            .add_delete()
    }
}
