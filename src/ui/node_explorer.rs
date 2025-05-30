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
        context: &mut Context,
        vctx: ViewContext,
        ui: &mut Ui,
        ids: &Vec<u64>,
        selected: Option<u64>,
    ) -> Result<Option<u64>, ExpressionError> {
        let mut new_selected: Option<u64> = None;
        ui.push_id(vctx.id, |ui| {
            let nodes = ids
                .into_iter()
                .filter_map(|id| context.get_by_id::<T>(*id).ok())
                .collect_vec();
            let mut table = nodes
                .table()
                .column(
                    "node",
                    |context, ui, node| {
                        if node
                            .view_node(
                                vctx.selected(selected.is_some_and(|id| id == node.id())),
                                context,
                                ui,
                            )
                            .title_clicked
                        {
                            new_selected = Some(node.id());
                        }
                    },
                    |_, node| node.id().into(),
                )
                .column(
                    "rating",
                    move |context, ui, node| {
                        node.node_view_rating(vctx, context, ui);
                    },
                    |_, node| node.node_rating().unwrap_or_default().into(),
                );
            if let Some((is_parent, id)) = vctx.link_rating {
                table = table.column(
                    "link rating",
                    move |context, ui, node| {
                        node.node_view_link_rating(vctx, context, ui, is_parent, id);
                    },
                    move |context, node| {
                        node.node_link_rating(context, is_parent, id)
                            .unwrap_or_default()
                            .0
                            .into()
                    },
                )
            }
            table.ui(context, ui);
        });
        Ok(new_selected)
    }
}

pub struct NodeExplorerPlugin;

#[derive(Resource, Clone, Default)]
pub struct NodeExplorerData {
    selected: Option<u64>,
    selected_kind: NodeKind,
    selected_ids: Vec<u64>,
    children: HashMap<NodeKind, Vec<u64>>,
    parents: HashMap<NodeKind, Vec<u64>>,
}

impl Plugin for NodeExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Explorer), |world: &mut World| {
            let kind = NodeKind::NHouse;
            let mut ned = NodeExplorerData {
                selected_kind: kind,
                ..default()
            };
            ned.selected_ids = kind.query_all_ids(world);
            world.insert_resource(ned);
        });
    }
}

impl NodeExplorerPlugin {
    fn select_id(
        context: &mut Context,
        ned: &mut NodeExplorerData,
        id: u64,
    ) -> Result<(), ExpressionError> {
        let kind = context.get_by_id::<NodeState>(id)?.kind;
        Self::select_kind(context.world_mut()?, ned, kind);
        ned.selected = Some(id);
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
    fn select_kind(world: &mut World, ned: &mut NodeExplorerData, kind: NodeKind) {
        ned.selected_kind = kind;
        ned.selected_ids = kind.query_all_ids(world);
        ned.children.clear();
        ned.parents.clear();
        ned.selected = None;
    }
    pub fn pane_selected(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let mut ned = world
            .remove_resource::<NodeExplorerData>()
            .to_e_not_found()?;
        let r = Context::from_world_r(world, |context| {
            let mut kind = ned.selected_kind;
            if Selector::new("kind").ui_enum(&mut kind, ui) {
                Self::select_kind(context.world_mut()?, &mut ned, kind);
            }
            if let Some(selected) = ned.selected {
                if format!("[red delete] #{selected}")
                    .cstr()
                    .button(ui)
                    .clicked()
                {
                    cn().reducers.on_content_delete_node(|e, _| {
                        e.event.notify_error();
                    });
                    cn().reducers.content_delete_node(selected).unwrap();
                }
            }
            if let Some(selected) = kind.show_explorer(
                context,
                ViewContext::new(ui).one_line(true),
                ui,
                &ned.selected_ids,
                ned.selected,
            )? {
                Self::select_id(context, &mut ned, selected)?;
            }
            Ok(())
        });
        world.insert_resource(ned);
        r
    }
    pub fn pane_node(ui: &mut Ui, world: &mut World) -> Result<(), ExpressionError> {
        let ned = world.get_resource::<NodeExplorerData>().to_e_not_found()?;
        let Some(id) = ned.selected else {
            return Ok(());
        };
        Context::from_world_r(world, |context| {
            format!("[tl #]{id} [tl e:]{}", context.entity(id)?)
                .cstr()
                .label(ui);
            let ns = context.get_by_id::<NodeState>(id)?;
            match ns.kind {
                NodeKind::NHouse => context
                    .get_by_id::<NHouse>(id)?
                    .tag_card(default(), context, ui)
                    .ui(ui),
                NodeKind::NAbilityMagic => context
                    .get_by_id::<NAbilityMagic>(id)?
                    .tag_card(default(), context, ui)
                    .ui(ui),
                NodeKind::NStatusMagic => context
                    .get_by_id::<NStatusMagic>(id)?
                    .tag_card(default(), context, ui)
                    .ui(ui),
                NodeKind::NFusion => context
                    .get_by_id::<NFusion>(id)?
                    .show_card(context, ui)
                    .ui(ui),
                NodeKind::NUnit => context
                    .get_by_id::<NUnit>(id)?
                    .tag_card(default(), context, ui)
                    .ui(ui),
                NodeKind::NRepresentation => {
                    context.get_by_id::<NRepresentation>(id)?.view(
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
            .remove_resource::<NodeExplorerData>()
            .to_e_not_found()?;
        Context::from_world_r(world, |context| {
            let mut selected: Option<u64> = None;
            let ids = if parents { &ned.parents } else { &ned.children };
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
            if let Some(selected) = ned.selected {
                vctx = vctx.link_rating(!parents, selected);
            }

            if selected_kind == NodeKind::None {
                for (kind, ids) in ids {
                    ui.vertical_centered_justified(|ui| {
                        kind.cstr_c(ui.visuals().weak_text_color()).label(ui);
                    });
                    if let Some(id) = kind.show_explorer(context, vctx, ui, ids, ned.selected)? {
                        selected = Some(id);
                    }
                }
            } else {
                let all_ids = selected_kind.query_all_ids(context.world_mut()?);
                if let Some(id) =
                    selected_kind.show_explorer(context, vctx, ui, &all_ids, ned.selected)?
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
