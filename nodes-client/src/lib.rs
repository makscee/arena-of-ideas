// Base types are imported in the generated code

// Client-specific traits and types
pub trait ClientNode {
    fn sync_from_server(&mut self, data: &[u8]) -> Result<(), ClientSyncError>;
    fn prepare_for_render(&self) -> RenderData;
}

#[derive(Debug)]
pub struct ClientSyncError {
    pub message: String,
}

#[derive(Debug, Default)]
pub struct RenderData {
    pub position: (f32, f32),
    pub visible: bool,
}

// Include client implementations generated at build time
include!(concat!(env!("OUT_DIR"), "/client_impls.rs"));

// Re-export the nodes! macro with client implementation
#[macro_export]
macro_rules! nodes {
    (client) => {
        // Re-export all base types and client implementations
        pub use nodes_client::*;
    };
}
