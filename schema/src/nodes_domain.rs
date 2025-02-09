use strum_macros::EnumIter;

#[derive(Clone, Copy, EnumIter, PartialEq, Eq, Hash)]
pub enum NodeDomain {
    World,
    Match,
    Core,
}
