use super::*;

#[derive(Default)]
pub struct SacrificeData {
    pub marked_units: HashSet<legion::Entity>,
    pub ranks_sum: usize,
}
