use super::*;
pub trait EntityExt {
    fn id(self, context: &ClientContext) -> NodeResult<u64>;
    fn get_children(self, context: &ClientContext) -> Result<Vec<Entity>, NodeError>;
    fn get_children_recursive(self, context: &ClientContext) -> Result<Vec<Entity>, NodeError>;
    fn get_parents(self, context: &ClientContext) -> Result<Vec<Entity>, NodeError>;
    fn to_value(self) -> VarValue;
}

impl EntityExt for Entity {
    fn id(self, context: &ClientContext) -> NodeResult<u64> {
        context.id(self)
    }
    fn get_children(self, context: &ClientContext) -> Result<Vec<Entity>, NodeError> {
        context.ids_to_entities(context.children(self.id(context)?))
    }
    fn get_children_recursive(self, context: &ClientContext) -> Result<Vec<Entity>, NodeError> {
        context.ids_to_entities(context.children_recursive(self.id(context)?))
    }
    fn get_parents(self, context: &ClientContext) -> Result<Vec<Entity>, NodeError> {
        context.ids_to_entities(context.parents(self.id(context)?))
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
