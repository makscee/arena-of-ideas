// Base types are imported in the generated code

// Server-specific traits and types
pub trait ServerNode {
    fn serialize_for_client(&self) -> Vec<u8>;
    fn update_from_client(&mut self, data: &[u8]) -> Result<(), ServerUpdateError>;
    fn authorize_access(&self, user_id: u64) -> bool;
    fn get_dependencies(&self) -> Vec<u64>;
}

pub trait DatabaseNode {
    fn table_name() -> &'static str;
    fn primary_key(&self) -> u64;
    fn save(&self) -> Result<(), DatabaseError>;
    fn load(id: u64) -> Result<Self, DatabaseError>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct ServerPersistError {
    pub message: String,
}

#[derive(Debug)]
pub struct ServerValidationError {
    pub message: String,
}

#[derive(Debug)]
pub struct ServerUpdateError {
    pub message: String,
}

#[derive(Debug)]
pub struct DatabaseError {
    pub message: String,
}

// Include server implementations generated at build time
include!(concat!(env!("OUT_DIR"), "/server_impls.rs"));

// Re-export the nodes! macro with server implementation
#[macro_export]
macro_rules! nodes {
    (server) => {
        // Re-export all base types and server implementations
        pub use nodes_server::*;
    };
}
