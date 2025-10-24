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
        self.ok_or_else(|| NodeError::var_not_found(f))
    }
}

impl ToEParam<Entity, u64> for Option<u64> {
    fn to_e(self, f: Entity) -> NodeResult<u64> {
        self.ok_or_else(|| NodeError::id_not_found(f.index(), f.generation().to_bits()))
    }
}

impl ToEParam<u64, Entity> for Option<Entity> {
    fn to_e(self, f: u64) -> NodeResult<Entity> {
        self.ok_or_else(|| NodeError::entity_not_found(f))
    }
}

impl<'a> ToE<&'a World> for Option<&'a World> {
    fn to_e(self) -> NodeResult<&'a World> {
        self.ok_or_else(|| NodeError::custom("World not found"))
    }
}

impl<'a> ToE<&'a mut World> for Option<&'a mut World> {
    fn to_e(self) -> NodeResult<&'a mut World> {
        self.ok_or_else(|| NodeError::custom("World not found"))
    }
}

impl<T> ToENotFound<T> for Option<T> {
    fn to_e_not_found(self) -> NodeResult<T> {
        self.ok_or_else(|| NodeError::not_found_generic(type_name_short::<T>()))
    }
}

pub trait ResultExt<T> {
    fn to_node_result(self) -> NodeResult<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn to_node_result(self) -> NodeResult<T> {
        self.map_err(|e| NodeError::custom(e.to_string()))
    }
}
