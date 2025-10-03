use super::*;
pub trait EntityExt {
    fn id(self, context: &ClientContext) -> NodeResult<u64>;
    fn get_children(self, context: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn get_children_recursive(self, context: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn get_parents(self, context: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn get_parents_recursive(self, context: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn first_parent(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn first_parent_recursive(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn first_child(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn first_child_recursive(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn collect_kind_children(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>>;
    fn collect_kind_children_recursive(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>>;
    fn collect_kind_parents(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>>;
    fn collect_kind_parents_recursive(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>>;
    fn to_value(self) -> VarValue;
}

impl EntityExt for Entity {
    fn id(self, context: &ClientContext) -> NodeResult<u64> {
        context.id(self)
    }
    fn get_children(self, context: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let child_ids = context.get_children(id)?;
        child_ids
            .into_iter()
            .map(|child_id| context.entity(child_id))
            .collect()
    }
    fn get_children_recursive(self, context: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let child_ids = context.children_recursive(id)?;
        child_ids
            .into_iter()
            .map(|child_id| context.entity(child_id))
            .collect()
    }
    fn get_parents(self, context: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let parent_ids = context.get_parents(id)?;
        parent_ids
            .into_iter()
            .map(|parent_id| context.entity(parent_id))
            .collect()
    }
    fn get_parents_recursive(self, context: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let parent_ids = context.parents_recursive(id)?;
        parent_ids
            .into_iter()
            .map(|parent_id| context.entity(parent_id))
            .collect()
    }
    fn first_parent(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(context)?;
        let parent_id = context.first_parent(id, kind)?;
        context.entity(parent_id)
    }
    fn first_parent_recursive(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(context)?;
        let parent_id = context.first_parent_recursive(id, kind)?;
        context.entity(parent_id)
    }
    fn first_child(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(context)?;
        let child_id = context.first_child(id, kind)?;
        context.entity(child_id)
    }
    fn first_child_recursive(self, context: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(context)?;
        let child_id = context.first_child_recursive(id, kind)?;
        context.entity(child_id)
    }
    fn collect_kind_children(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let child_ids = context.collect_kind_children(id, kind)?;
        child_ids
            .into_iter()
            .map(|child_id| context.entity(child_id))
            .collect()
    }
    fn collect_kind_children_recursive(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let child_ids = context.collect_kind_children_recursive(id, kind)?;
        child_ids
            .into_iter()
            .map(|child_id| context.entity(child_id))
            .collect()
    }
    fn collect_kind_parents(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let parent_ids = context.collect_kind_parents(id, kind)?;
        parent_ids
            .into_iter()
            .map(|parent_id| context.entity(parent_id))
            .collect()
    }
    fn collect_kind_parents_recursive(
        self,
        context: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>> {
        let id = self.id(context)?;
        let parent_ids = context.collect_kind_parents_recursive(id, kind)?;
        parent_ids
            .into_iter()
            .map(|parent_id| context.entity(parent_id))
            .collect()
    }
    fn to_value(self) -> VarValue {
        VarValue::Entity(self.to_bits())
    }
}

pub trait EntityVecVarValue {
    fn vec_to_value(self) -> VarValue;
}

impl EntityVecVarValue for Vec<Entity> {
    fn vec_to_value(self) -> VarValue {
        self.into_iter().map(|e| e.to_value()).collect_vec().into()
    }
}
