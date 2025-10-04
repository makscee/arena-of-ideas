use super::*;
pub trait EntityExt {
    fn id(self, ctx: &ClientContext) -> NodeResult<u64>;
    fn get_children(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn get_children_recursive(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn get_parents(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn get_parents_recursive(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>>;
    fn first_parent(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn first_parent_recursive(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn first_child(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn first_child_recursive(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity>;
    fn collect_kind_children(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Vec<Entity>>;
    fn collect_kind_children_recursive(
        self,
        ctx: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>>;
    fn collect_kind_parents(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Vec<Entity>>;
    fn collect_kind_parents_recursive(
        self,
        ctx: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>>;
    fn to_value(self) -> VarValue;
}

impl EntityExt for Entity {
    fn id(self, ctx: &ClientContext) -> NodeResult<u64> {
        ctx.id(self)
    }
    fn get_children(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let child_ids = ctx.get_children(id)?;
        child_ids
            .into_iter()
            .map(|child_id| ctx.entity(child_id))
            .collect()
    }
    fn get_children_recursive(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let child_ids = ctx.children_recursive(id)?;
        child_ids
            .into_iter()
            .map(|child_id| ctx.entity(child_id))
            .collect()
    }
    fn get_parents(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let parent_ids = ctx.get_parents(id)?;
        parent_ids
            .into_iter()
            .map(|parent_id| ctx.entity(parent_id))
            .collect()
    }
    fn get_parents_recursive(self, ctx: &ClientContext) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let parent_ids = ctx.parents_recursive(id)?;
        parent_ids
            .into_iter()
            .map(|parent_id| ctx.entity(parent_id))
            .collect()
    }
    fn first_parent(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(ctx)?;
        let parent_id = ctx.first_parent(id, kind)?;
        ctx.entity(parent_id)
    }
    fn first_parent_recursive(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(ctx)?;
        let parent_id = ctx.first_parent_recursive(id, kind)?;
        ctx.entity(parent_id)
    }
    fn first_child(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(ctx)?;
        let child_id = ctx.first_child(id, kind)?;
        ctx.entity(child_id)
    }
    fn first_child_recursive(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Entity> {
        let id = self.id(ctx)?;
        let child_id = ctx.first_child_recursive(id, kind)?;
        ctx.entity(child_id)
    }
    fn collect_kind_children(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let child_ids = ctx.collect_kind_children(id, kind)?;
        child_ids
            .into_iter()
            .map(|child_id| ctx.entity(child_id))
            .collect()
    }
    fn collect_kind_children_recursive(
        self,
        ctx: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let child_ids = ctx.collect_kind_children_recursive(id, kind)?;
        child_ids
            .into_iter()
            .map(|child_id| ctx.entity(child_id))
            .collect()
    }
    fn collect_kind_parents(self, ctx: &ClientContext, kind: NodeKind) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let parent_ids = ctx.collect_kind_parents(id, kind)?;
        parent_ids
            .into_iter()
            .map(|parent_id| ctx.entity(parent_id))
            .collect()
    }
    fn collect_kind_parents_recursive(
        self,
        ctx: &ClientContext,
        kind: NodeKind,
    ) -> NodeResult<Vec<Entity>> {
        let id = self.id(ctx)?;
        let parent_ids = ctx.collect_kind_parents_recursive(id, kind)?;
        parent_ids
            .into_iter()
            .map(|parent_id| ctx.entity(parent_id))
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
