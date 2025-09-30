use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkState<T> {
    Loaded(Box<T>),
    Id(u64),
    None,
    Unknown,
}

pub trait Link<T> {
    fn state(&self) -> &LinkState<T>;
    fn state_mut(&mut self) -> &mut LinkState<T>;

    fn new_loaded(value: T) -> Self;
    fn new_id(id: u64) -> Self;
    fn none() -> Self;
    fn unknown() -> Self;

    fn get<'a>(&'a self) -> Option<&'a T> {
        match self.state() {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self) -> Option<&'a mut T> {
        match self.state_mut() {
            LinkState::Loaded(val) => Some(val),
            _ => None,
        }
    }

    fn id(&self) -> Option<u64> {
        match self.state() {
            LinkState::Id(id) => Some(*id),
            _ => None,
        }
    }

    fn is_loaded(&self) -> bool {
        matches!(self.state(), LinkState::Loaded(_))
    }

    fn is_none(&self) -> bool {
        matches!(self.state(), LinkState::None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component<T> {
    state: LinkState<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owned<T> {
    state: LinkState<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ref<T> {
    state: LinkState<T>,
}

impl<T> Link<T> for Component<T> {
    fn state(&self) -> &LinkState<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkState<T> {
        &mut self.state
    }

    fn new_loaded(value: T) -> Self {
        Self {
            state: LinkState::Loaded(Box::new(value)),
        }
    }

    fn new_id(id: u64) -> Self {
        Self {
            state: LinkState::Id(id),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkState::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkState::Unknown,
        }
    }
}

impl<T> Link<T> for Owned<T> {
    fn state(&self) -> &LinkState<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkState<T> {
        &mut self.state
    }

    fn new_loaded(value: T) -> Self {
        Self {
            state: LinkState::Loaded(Box::new(value)),
        }
    }

    fn new_id(id: u64) -> Self {
        Self {
            state: LinkState::Id(id),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkState::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkState::Unknown,
        }
    }
}

impl<T> Link<T> for Ref<T> {
    fn state(&self) -> &LinkState<T> {
        &self.state
    }

    fn state_mut(&mut self) -> &mut LinkState<T> {
        &mut self.state
    }

    fn new_loaded(value: T) -> Self {
        Self {
            state: LinkState::Loaded(Box::new(value)),
        }
    }

    fn new_id(id: u64) -> Self {
        Self {
            state: LinkState::Id(id),
        }
    }

    fn none() -> Self {
        Self {
            state: LinkState::None,
        }
    }

    fn unknown() -> Self {
        Self {
            state: LinkState::Unknown,
        }
    }
}

impl<T> Default for Component<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<T> Default for Owned<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

impl<T> Default for Ref<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

// IntoIterator implementations for Vec types only
impl<'a, T> IntoIterator for &'a Owned<Vec<T>> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().map(|vec| vec.iter()).into_iter().flatten()
    }
}

impl<'a, T> IntoIterator for &'a Component<Vec<T>> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().map(|vec| vec.iter()).into_iter().flatten()
    }
}

impl<'a, T> IntoIterator for &'a Ref<Vec<T>> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<std::slice::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().map(|vec| vec.iter()).into_iter().flatten()
    }
}

// IntoIterator implementations for Option types only
impl<'a, T> IntoIterator for &'a Owned<Option<T>> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().and_then(|opt| opt.as_ref()).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Component<Option<T>> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().and_then(|opt| opt.as_ref()).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Ref<Option<T>> {
    type Item = &'a T;
    type IntoIter = std::option::IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.get().and_then(|opt| opt.as_ref()).into_iter()
    }
}

// Extension trait for iteration functionality that avoids trait conflicts
pub trait LinkIterable<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_>;
}

impl<T> LinkIterable<T> for Owned<Vec<T>> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<T> LinkIterable<T> for Component<Vec<T>> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<T> LinkIterable<T> for Ref<Vec<T>> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.get() {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }
    }
}

impl<T> LinkIterable<T> for Owned<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().into_iter())
    }
}

impl<T> LinkIterable<T> for Component<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().into_iter())
    }
}

impl<T> LinkIterable<T> for Ref<T> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().into_iter())
    }
}

impl<T> LinkIterable<T> for Owned<Option<T>> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().and_then(|opt| opt.as_ref()).into_iter())
    }
}

impl<T> LinkIterable<T> for Component<Option<T>> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().and_then(|opt| opt.as_ref()).into_iter())
    }
}

impl<T> LinkIterable<T> for Ref<Option<T>> {
    fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(self.get().and_then(|opt| opt.as_ref()).into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration_functionality() {
        // Test Vec iteration
        let vec_data = vec![1, 2, 3];
        let owned_vec: Owned<Vec<i32>> = Owned::new_loaded(vec_data);

        let mut collected = Vec::new();
        for item in &owned_vec {
            collected.push(*item);
        }
        assert_eq!(collected, vec![1, 2, 3]);

        // Test single item iteration
        let owned_single: Owned<i32> = Owned::new_loaded(42);
        let mut collected = Vec::new();
        for item in owned_single.iter() {
            collected.push(*item);
        }
        assert_eq!(collected, vec![42]);

        // Test Option iteration with Some
        let owned_option: Owned<Option<i32>> = Owned::new_loaded(Some(99));
        let mut collected = Vec::new();
        for item in &owned_option {
            collected.push(*item);
        }
        assert_eq!(collected, vec![99]);

        // Test Option iteration with None
        let owned_option_none: Owned<Option<i32>> = Owned::new_loaded(None);
        let mut collected = Vec::new();
        for item in &owned_option_none {
            collected.push(*item);
        }
        assert_eq!(collected, Vec::<i32>::new());

        // Test empty Vec
        let owned_empty_vec: Owned<Vec<i32>> = Owned::new_loaded(vec![]);
        let mut collected = Vec::new();
        for item in &owned_empty_vec {
            collected.push(*item);
        }
        assert_eq!(collected, Vec::<i32>::new());

        // Test with LinkIterable trait
        use LinkIterable;
        let vec_data = vec![10, 20, 30];
        let owned_vec: Owned<Vec<i32>> = Owned::new_loaded(vec_data);
        let collected: Vec<&i32> = owned_vec.iter().collect();
        assert_eq!(collected, vec![&10, &20, &30]);
    }
}
