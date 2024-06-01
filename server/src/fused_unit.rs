use super::*;

#[derive(SpacetimeType)]
pub struct FusedUnit {
    id: u64,
    bases: Vec<String>,
    triggers: Vec<u32>,
    targets: Vec<u32>,
    effects: Vec<u32>,
    stacks: u32,
}
