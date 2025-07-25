use std::marker::PhantomData;

use bevy_egui::egui::Grid;

use super::*;

pub struct NodesListWidget<T: NodeViewFns> {
    pd: PhantomData<T>,
}

impl<T: NodeViewFns> NodesListWidget<T> {
    pub fn new() -> NodesListWidget<T> {
        Self { pd: PhantomData }
    }
    pub fn ui(
        &mut self,
        context: &Context,
        vctx: ViewContext,
        ui: &mut Ui,
        ids: &Vec<u64>,
        selected: Option<u64>,
    ) -> Result<Option<u64>, ExpressionError> {
        let mut new_selected: Option<u64> = None;
        ui.push_id(vctx.id, |ui| {
            let mut nodes = ids
                .into_iter()
                .filter_map(|id| context.get_by_id::<T>(*id).ok())
                .collect_vec();

            // Sort by rating (descending) then by node_id (ascending)
            nodes.sort_by(|a, b| {
                let rating_a = a.node_rating().unwrap_or_default();
                let rating_b = b.node_rating().unwrap_or_default();
                match rating_b.cmp(&rating_a) {
                    // descending rating
                    std::cmp::Ordering::Equal => a.id().cmp(&b.id()), // ascending id
                    other => other,
                }
            });
            let mut table = nodes.table().column(
                "r",
                move |context, ui, node, _value| {
                    node.node_view_rating(vctx, context, ui);
                    Ok(())
                },
                |_, node| Ok(VarValue::i32(node.node_rating().unwrap_or_default())),
            );
            if let Some((is_parent, id)) = vctx.link_rating {
                table = table.column(
                    "lr",
                    move |context, ui, node, _value| {
                        node.node_view_link_rating(vctx, context, ui, is_parent, id);
                        Ok(())
                    },
                    move |context, node| {
                        Ok(VarValue::i32(
                            node.node_link_rating(context, is_parent, id)
                                .unwrap_or_default()
                                .0,
                        ))
                    },
                );
            }
            table = table
                .column(
                    "owner",
                    |_, ui, node, _value| {
                        node.owner().cstr_s(CstrStyle::Small).label(ui);
                        Ok(())
                    },
                    |_, node| Ok(VarValue::u64(node.owner())),
                )
                .column(
                    "node",
                    |context, ui, node, _value| {
                        ui.horizontal(|ui| {
                            let response = node.view_title(
                                vctx.selected(selected.is_some_and(|id| id == node.id())),
                                context,
                                ui,
                            );
                            response.bar_menu(|ui| {
                                if "open in inspector".cstr().button(ui).clicked() {
                                    new_selected = Some(node.id());
                                    ui.close_menu();
                                }
                            });
                            node.view_data(vctx.one_line(true), context, ui);
                        });
                        Ok(())
                    },
                    |_, node| Ok(VarValue::String(node.get_data())),
                )
                .column_remainder();
            table.ui(context, ui);
        });
        Ok(new_selected)
    }
}

pub struct NodeExplorerPlugin;

impl Plugin for NodeExplorerPlugin {
    fn build(&self, _app: &mut App) {
        // No longer needed - using shared initialization from node_explorer_new
    }
}

impl NodeExplorerPlugin {
    fn select_id(
        context: &mut Context,
        ned: &mut NodeExplorerDataNew,
        id: u64,
    ) -> Result<(), ExpressionError> {
        let kind = context.get_by_id::<NodeState>(id)?.kind;
        Self::select_kind(context.world_mut()?, ned, kind);
        ned.inspected_node = Some(id);
        for child in context.children(id) {
            let kind = child.kind()?;
            ned.children.entry(kind).or_default().push(child);
        }
        for parent in context.parents(id) {
            let kind = parent.kind()?;
            ned.parents.entry(kind).or_default().push(parent);
        }
        Ok(())
    }
    fn select_kind(world: &mut World, ned: &mut NodeExplorerDataNew, kind: NodeKind) {
        ned.selected_kind = kind;
        let filter_ids = ned.owner_filter.ids();
        ned.selected_ids = kind
            .query_all_ids(world)
            .into_iter()
            .filter(|id| {
                id.get_node()
                    .is_some_and(|node| filter_ids.is_empty() || filter_ids.contains(&node.owner))
            })
            .collect();
        ned.children.clear();
        ned.parents.clear();
        ned.inspected_node = None;
    }
    pub fn pane_selected(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut ned = world
            .remove_resource::<NodeExplorerDataNew>()
            .to_e_not_found()?;
        let r = Context::from_world_r(world, |context| {
            let kind = ned.selected_kind;
            if let Some(selected) = kind.show_explorer(
                context,
                ViewContext::new(ui).one_line(true),
                ui,
                &ned.selected_ids,
                ned.inspected_node,
            )? {
                Self::select_id(context, &mut ned, selected)?;
            }
            Ok(())
        });
        world.insert_resource(ned);
        r
    }
    pub fn pane_node(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let ned = world
            .get_resource::<NodeExplorerDataNew>()
            .to_e_not_found()?;
        let Some(id) = ned.inspected_node else {
            return Ok(());
        };
        let node = id.get_node().to_e_not_found()?;
        Context::from_world_r(world, |context| {
            format!(
                "[tw #]{id} [tw e:]{} [tw owner:] {}",
                context.entity(id)?,
                node.owner
            )
            .cstr()
            .label(ui);

            if format!("[red delete] #{id}").cstr().button(ui).clicked() {
                cn().reducers.admin_delete_node(id).unwrap();
            }
            let ns = context.get_by_id::<NodeState>(id)?;
            match ns.kind {
                NodeKind::NHouse => context
                    .get_by_id::<NHouse>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NActionAbility => context
                    .get_by_id::<NActionAbility>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NStatusAbility => context
                    .get_by_id::<NStatusAbility>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NFusion => context
                    .get_by_id::<NFusion>(id)?
                    .show_card(context, ui)
                    .ui(ui),
                NodeKind::NUnit => context
                    .get_by_id::<NUnit>(id)?
                    .see(context)
                    .tag_card(ui)
                    .ui(ui),
                NodeKind::NUnitRepresentation => {
                    context.get_by_id::<NUnitRepresentation>(id)?.view(
                        ViewContext::new(ui),
                        context,
                        ui,
                    );
                }
                _ => {}
            }
            Grid::new("vars").show(ui, |ui| {
                for (var, state) in &ns.vars {
                    var.cstr().label(ui);
                    state.value.cstr().label(ui);
                    ui.end_row();
                }
            });
            Ok(())
        })
    }
    pub fn pane_parents(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Self::pane_relations(ui, world, true)
    }
    pub fn pane_children(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        Self::pane_relations(ui, world, false)
    }
    fn pane_relations(
        ui: &mut Ui,
        world: &mut World,
        parents: bool,
    ) -> Result<(), ExpressionError> {
        let mut ned = world
            .remove_resource::<NodeExplorerDataNew>()
            .to_e_not_found()?;
        Context::from_world_r(world, |context| {
            let mut selected: Option<u64> = None;
            let ids = if parents { &ned.parents } else { &ned.children };
            if ids.is_empty() {
                let label_text = if parents { "No parents" } else { "No children" };
                ui.label(label_text);
                return Ok(());
            }
            let mut selected_kind = ui
                .ctx()
                .data_mut(|w| w.get_temp_mut_or_default::<NodeKind>(ui.id()).clone());
            if EnumSwitcher::new().show_iter(
                &mut selected_kind,
                [NodeKind::None].iter().chain(ids.keys()),
                ui,
            ) {
                ui.ctx().data_mut(|w| w.insert_temp(ui.id(), selected_kind));
            }
            let mut vctx = ViewContext::new(ui).one_line(true);
            if let Some(selected) = ned.inspected_node {
                vctx = vctx.link_rating(!parents, selected);
            }
            if selected_kind == NodeKind::None {
                for (kind, ids) in ids {
                    ui.vertical_centered_justified(|ui| {
                        kind.cstr_c(ui.visuals().weak_text_color()).label(ui);
                    });
                    if let Some(id) =
                        kind.show_explorer(context, vctx, ui, ids, ned.inspected_node)?
                    {
                        selected = Some(id);
                    }
                }
            } else {
                let all_ids = selected_kind.query_all_ids(context.world_mut()?);
                if let Some(id) =
                    selected_kind.show_explorer(context, vctx, ui, &all_ids, ned.inspected_node)?
                {
                    selected = Some(id);
                }
            }
            if let Some(selected) = selected {
                Self::select_id(context, &mut ned, selected)?;
                ui.ctx().data_mut(|w| w.remove_by_type::<NodeKind>());
            }
            Ok(())
        })?;
        world.insert_resource(ned);
        Ok(())
    }
}
