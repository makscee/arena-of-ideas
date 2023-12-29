mod tower;
mod user;
mod hero;

use anyhow::Context;

use spacetimedb::SpacetimeType;
use spacetimedb::{spacetimedb, Identity, ReducerContext};
