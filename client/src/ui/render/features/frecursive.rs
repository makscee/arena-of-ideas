use super::*;

/// Feature for types that can be rendered recursively with breadcrumb navigation
pub trait FRecursiveNodeEdit: FEdit + FTitle + ClientNode + Clone + Debug {
    /// Render the node recursively, handling breadcrumbs and inspection state
    fn render_recursive_edit(&mut self, ui: &mut Ui, ctx: &ClientContext) -> bool {
        let node_id = self.id();
        ui.set_edit_context(node_id);
        let result = render_node_field_recursive_with_path(ui, "root", self, &mut vec![], ctx);
        ui.clear_edit_context();
        result
    }

    /// Render linked fields for the current node
    fn render_linked_fields(
        &mut self,
        ui: &mut Ui,
        ctx: &ClientContext,
        breadcrumb_path: &mut Vec<NodeBreadcrumb>,
    ) -> bool;

    /// Search recursively through linked fields for inspected node
    fn render_recursive_search(
        &mut self,
        ui: &mut Ui,
        ctx: &ClientContext,
        breadcrumb_path: &mut Vec<NodeBreadcrumb>,
    ) -> bool;
}

pub trait NodeKindCompact {
    fn is_compact(self) -> bool;
}
impl NodeKindCompact for NodeKind {
    fn is_compact(self) -> bool {
        match self {
            Self::NUnit
            | Self::NUnitBehavior
            | Self::NAbilityEffect
            | Self::NStatusBehavior
            | Self::NHouseColor
            | Self::NUnitStats => true,
            _ => false,
        }
    }
}

/// Render breadcrumbs from a path vector
pub fn render_breadcrumbs(ui: &mut Ui, breadcrumb_path: &[NodeBreadcrumb]) -> Option<u64> {
    if breadcrumb_path.is_empty() {
        return None;
    }

    let mut clicked_id = None;

    ScrollArea::horizontal()
        .stick_to_right(true)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (i, crumb) in breadcrumb_path.iter().enumerate() {
                    if i > 0 {
                        ui.label(" > ");
                    }

                    let is_last = i == breadcrumb_path.len() - 1;
                    let label = if let Some(ref field_name) = crumb.field_name {
                        format!("{}", field_name)
                    } else {
                        format!("{:?}", crumb.kind)
                    };

                    if is_last {
                        ui.strong(&label);
                    } else {
                        if ui.link(&label).clicked() {
                            clicked_id = Some(crumb.id);
                        }
                    }
                }
            });
        });
    ui.separator();

    clicked_id
}

/// Helper trait for rendering node link fields
pub trait NodeLinkRender {
    /// Render a single link field (Component or Owned)
    fn render_single_link<L, T>(
        &mut self,
        field_name: &str,
        link: &mut L,
        owner_id: u64,
        ctx: &ClientContext,
    ) -> bool
    where
        L: SingleLink<T>,
        T: FRecursiveNodeEdit + Node + Default;

    /// Render a multiple link field
    fn render_multiple_link<L, T>(
        &mut self,
        field_name: &str,
        link: &mut L,
        owner_id: u64,
        ctx: &ClientContext,
    ) -> bool
    where
        L: MultipleLink<T>,
        T: FRecursiveNodeEdit + Node + Default;
}

impl NodeLinkRender for Ui {
    fn render_single_link<L, T>(
        &mut self,
        field_name: &str,
        link: &mut L,
        owner_id: u64,
        ctx: &ClientContext,
    ) -> bool
    where
        L: SingleLink<T>,
        T: FRecursiveNodeEdit + Node + Default,
    {
        let mut need_remove = false;
        let changed = if let Ok(loaded) = link.get_mut() {
            let mut changed = false;
            self.horizontal(|ui| {
                changed |= render_node_menu(ui, loaded);
                if format!(
                    "{field_name}: [tw {}] {}",
                    loaded.kind(),
                    loaded.title(&EMPTY_CONTEXT)
                )
                .button(ui)
                .clicked()
                {
                    ui.set_inspected_node(loaded.id());
                }
                if "[red [b -]]"
                    .cstr()
                    .button(ui)
                    .on_hover_text("Delete Node")
                    .clicked()
                {
                    need_remove = true;
                }
            });
            changed
        } else {
            if self.button(format!("➕ Add {}", field_name)).clicked() {
                let mut new_node = T::default();
                new_node.set_id(next_id());
                new_node.set_owner(owner_id);
                link.set_loaded(new_node);
                return true;
            }
            false
        };
        self.separator();
        if need_remove {
            *link = SingleLink::none(owner_id);
            true
        } else {
            changed
        }
    }

    fn render_multiple_link<L, T>(
        &mut self,
        field_name: &str,
        link: &mut L,
        owner_id: u64,
        ctx: &ClientContext,
    ) -> bool
    where
        L: MultipleLink<T>,
        T: FRecursiveNodeEdit + Node + Default,
    {
        let mut changed = false;

        self.vertical(|ui| {
            ui.label(format!("{}:", field_name));
            if let Ok(items) = link.get_mut() {
                let mut to_remove: Option<u64> = None;
                for (index, item) in items
                    .iter_mut()
                    .sorted_by_key(|i| {
                        i.get_var(VarName::index)
                            .ok()
                            .and_then(|v| v.get_i32().ok())
                            .unwrap_or(i.id() as i32)
                    })
                    .enumerate()
                {
                    ui.horizontal(|ui| {
                        changed |= render_node_menu(ui, item);
                        if format!(
                            "{field_name} #{index}: [tw {}] {}",
                            item.kind(),
                            item.title(&EMPTY_CONTEXT)
                        )
                        .button(ui)
                        .clicked()
                        {
                            ui.set_inspected_node(item.id());
                        }
                        if "[red [b -]]"
                            .cstr()
                            .button(ui)
                            .on_hover_text("Delete Node")
                            .clicked()
                        {
                            to_remove = Some(item.id());
                        }
                    });
                }
                if let Some(id) = to_remove {
                    items.retain(|i| i.id() != id);
                    changed = true;
                }
                if ui.button(format!("➕ Push to {}", field_name)).clicked() {
                    let mut new_item = T::default();
                    new_item.set_id(next_id());
                    new_item.set_owner(owner_id);
                    items.push(new_item);
                    changed = true;
                }
            } else {
                if ui
                    .button(format!("➕ Create {} list", field_name))
                    .clicked()
                {
                    link.set_loaded(vec![]);
                    changed = true;
                }
            }
        });
        self.separator();

        changed
    }
}

fn render_node_menu<T: FRecursiveNodeEdit>(ui: &mut Ui, node: &mut T) -> bool {
    let mut changed = false;
    let menu_resp = node
        .as_empty_mut()
        .with_menu()
        .add_copy()
        .add_paste()
        .add_action("Copy Full", |d, _| {
            clipboard_set(d.pack().to_string());
            Some(MenuAction::Copy)
        })
        .add_action("Paste Full", |_, _| {
            let pack = PackedNodes::from_string(&clipboard_get()?).ok()?;
            let unpack = T::unpack(&pack);
            Some(MenuAction::Paste(unpack.ok()?))
        })
        .add_action("📦 Open Publish Window", |d, _| {
            op(|world| {
                d.open_publish_window(world, None);
            });
            None
        })
        .add_submenu("⭐️ Load From Core", |ui, _, _| {
            match with_core_source(|ctx| {
                let mut selected = None;

                let nodes = ctx
                    .world_mut()?
                    .query::<&T>()
                    .iter(ctx.world()?)
                    .collect_vec();
                for n in nodes {
                    if ui
                        .menu_button(n.title(ctx).widget(1.0, ui.style()), |ui| {
                            if let Ok(kind) = n.kind().to_named() {
                                named_node_kind_match!(kind, {
                                    ctx.load::<NamedNodeType>(n.id())
                                        .unwrap()
                                        .as_card()
                                        .compose(ctx, ui);
                                });
                            }
                        })
                        .response
                        .clicked()
                    {
                        selected = Some(n.id());
                        ui.close_kind(egui::UiKind::Menu);
                    }
                }
                if let Some(selected) = selected {
                    return Ok(Some(dbg!(ctx.load::<T>(selected)?.load_all(ctx)?.take())));
                }
                Ok(None)
            }) {
                Ok(n) => {
                    if let Some(n) = n {
                        return Some(MenuAction::Paste(n));
                    }
                }
                Err(e) => e.log(),
            }
            None
        })
        .compose_with_menu(&EMPTY_CONTEXT, ui);
    if let Some(action) = menu_resp.action {
        if let MenuAction::Paste(pasted) = action {
            let id = node.id();
            *node = pasted.remap_ids();
            node.set_id(id);
            changed = true;
        }
    }
    changed
}

/// Main composition function that handles recursive rendering with breadcrumbs
pub fn render_node_field_recursive_with_path<T: FRecursiveNodeEdit>(
    ui: &mut Ui,
    field_name: &str,
    field_node: &mut T,
    breadcrumb_path: &mut Vec<NodeBreadcrumb>,
    ctx: &ClientContext,
) -> bool {
    let node_id = field_node.id();
    let node_kind = field_node.kind();
    let inspected = ui.inspected_node();
    breadcrumb_path.push(NodeBreadcrumb {
        id: node_id,
        kind: node_kind,
        field_name: Some(field_name.to_string()),
    });

    let changed = if inspected == Some(node_id) {
        if let Some(clicked_id) = render_breadcrumbs(ui, &breadcrumb_path) {
            ui.set_inspected_node(clicked_id);
        }
        let mut changed = false;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                changed |= render_node_menu(ui, field_node);
                ui.separator();
                format!("[s #[tw {}]]", field_node.id()).label(ui);
                ui.separator();
                format!("[tw [b Owner:]] #[tw {}]", field_node.owner()).label(ui);
            });
        });
        ctx.exec_ref(|ctx| {
            ctx.set_owner(node_id);
            changed |= field_node.edit(ui, ctx).changed();
            Ok(())
        })
        .ui(ui);
        ui.separator();
        changed |= field_node.render_linked_fields(ui, ctx, breadcrumb_path);

        changed
    } else if inspected.is_none() {
        ui.set_inspected_node(node_id);
        false
    } else {
        ctx.exec_ref(|ctx| {
            ctx.set_owner(node_id);
            Ok(field_node.render_recursive_search(ui, ctx, breadcrumb_path))
        })
        .unwrap()
    };
    breadcrumb_path.pop();
    changed
}
