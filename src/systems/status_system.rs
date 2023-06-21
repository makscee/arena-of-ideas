use super::*;

pub struct StatusSystem {}

impl StatusSystem {
    pub fn unpack_into_state(state: &mut ContextState, statuses: &Vec<(String, i32)>) {
        for (name, charges) in statuses.iter() {
            *state.statuses.entry(name.to_owned()).or_default() += *charges;
            state.t += 1;
            state.status_change_t.insert(name.to_owned(), state.t);
        }
    }

    pub fn pack_state_into_vec(state: &ContextState) -> Vec<(String, i32)> {
        Vec::from_iter(
            state
                .statuses
                .clone()
                .into_iter()
                .sorted_by_key(|(name, _)| state.status_change_t.get(name).unwrap_or(&0)),
        )
    }
}
