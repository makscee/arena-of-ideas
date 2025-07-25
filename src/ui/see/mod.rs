mod builder;
mod card;
mod tag;
mod tag_card;
mod title;

use super::*;

pub use builder::*;
pub use card::*;
pub use tag::*;
pub use tag_card::*;
pub use title::*;

pub trait See: Sized {
    fn see<'a>(&'a self, ctx: &'a Context<'a>) -> SeeBuilder<'a, Self> {
        SeeBuilder::new(self, ctx)
    }
}

impl<T> See for T {}
