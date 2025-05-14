use std::marker::PhantomData;

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
        ui: &mut Ui,
        ids: &Vec<u64>,
        selected: Option<u64>,
    ) -> Result<Option<u64>, ExpressionError> {
        let mut vctx = ViewContext::new(ui).one_line(true);
        let mut new_selected: Option<u64> = None;
        for n in ids
            .into_iter()
            .filter_map(|id| context.get_by_id::<T>(*id).ok())
        {
            vctx.selected = selected.is_some_and(|id| id == n.id());
            if n.view_node(vctx, context, ui).title_clicked {
                new_selected = Some(n.id());
            }
        }
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
            if let Some(selected) =
                kind.show_explorer(context, ui, &ned.selected_ids, ned.selected)?
            {
                Self::select_id(context, &mut ned, selected)?;
            }
            Ok(())
        });
        world.insert_resource(ned);
        r
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
            for (kind, ids) in if parents { &ned.parents } else { &ned.children } {
                ui.vertical_centered_justified(|ui| {
                    kind.cstr().label(ui);
                });
                if let Some(id) = kind.show_explorer(context, ui, ids, ned.selected)? {
                    selected = Some(id);
                }
            }
            if let Some(selected) = selected {
                Self::select_id(context, &mut ned, selected)?;
            }
            Ok(())
        })?;
        world.insert_resource(ned);
        Ok(())
    }
}
