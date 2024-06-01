use super::*;

#[derive(SpacetimeType, Clone)]
pub struct FusedUnit {
    pub bases: Vec<String>,
    pub triggers: Vec<u32>,
    pub targets: Vec<u32>,
    pub effects: Vec<u32>,
    pub stacks: u32,
}
