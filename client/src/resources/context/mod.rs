mod client_source;
mod extensions;
mod node_maps;
mod relationships;
mod sources;

pub use client_source::*;
pub use extensions::*;
pub use node_maps::*;
pub use relationships::*;
pub use sources::*;

use crate::prelude::*;

pub type ClientContext<'w> = Context<ClientSource<'w>>;
pub const EMPTY_CONTEXT: ClientContext = Context::new(ClientSource::None);
