use super::*;

/// A Rhai script that can be compiled and executed to produce actions of type T
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RhaiScript<T> {
    pub code: String,
    pub description: String,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for RhaiScript<T> {
    fn default() -> Self {
        Self {
            code: String::new(),
            description: String::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> RhaiScript<T> {
    pub fn new(code: String) -> Self {
        Self {
            code,
            description: String::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn empty() -> Self {
        Self {
            code: String::new(),
            description: String::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Marker types for different action types that scripts can produce
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnitAction;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusAction;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbilityAction;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RhaiPainterAction;
