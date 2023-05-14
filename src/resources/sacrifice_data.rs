use super::*;

#[derive(Default)]
pub struct SacrificeData {
    pub marked_units: HashSet<legion::Entity>,
}
