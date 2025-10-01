use schema::{NodeError, NodeResult, type_name_short};

use super::*;

pub trait ToEParam<F, T> {
    fn to_e(self, f: F) -> NodeResult<T>;
}

pub trait ToE<T> {
    fn to_e(self) -> NodeResult<T>;
}

pub trait ToENotFound<T> {
    fn to_e_not_found(self) -> NodeResult<T>;
}

impl ToEParam<VarName, VarValue> for Option<VarValue> {
    fn to_e(self, f: VarName) -> NodeResult<VarValue> {
        self.ok_or_else(|| NodeError::VarNotFound(f))
    }
}

impl ToEParam<Entity, u64> for Option<u64> {
    fn to_e(self, f: Entity) -> NodeResult<u64> {
        self.ok_or_else(|| NodeError::IdNotFound(f.index(), f.generation()))
    }
}

impl ToEParam<u64, Entity> for Option<Entity> {
    fn to_e(self, f: u64) -> NodeResult<Entity> {
        self.ok_or_else(|| NodeError::EntityNotFound(f))
    }
}

impl<'a> ToE<&'a World> for Option<&'a World> {
    fn to_e(self) -> NodeResult<&'a World> {
        self.ok_or_else(|| NodeError::Custom("World not found".into()))
    }
}

impl<'a> ToE<&'a mut World> for Option<&'a mut World> {
    fn to_e(self) -> NodeResult<&'a mut World> {
        self.ok_or_else(|| NodeError::Custom("World not found".into()))
    }
}

impl<T> ToENotFound<T> for Option<T> {
    fn to_e_not_found(self) -> NodeResult<T> {
        self.ok_or_else(|| {
            NodeError::NotFoundGeneric(format!("Not found: {}", type_name_short::<T>()))
        })
    }
}

// Additional conversion traits for cleaner code
pub trait EntityToValue {
    fn to_value(self) -> VarValue;
}

impl EntityToValue for Entity {
    fn to_value(self) -> VarValue {
        VarValue::Entity(self.to_bits())
    }
}

pub trait ResultExt<T> {
    fn to_node_result(self) -> NodeResult<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn to_node_result(self) -> NodeResult<T> {
        self.map_err(|e| NodeError::Custom(e.to_string()))
    }
}
